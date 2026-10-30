use crate::{model::*,render::{pixmap_from_rgba,rgba}};
use anyhow::{bail,ensure,Result};
use base64::{engine::general_purpose::STANDARD,Engine};
use image::{ImageReader,DynamicImage,ImageFormat,Limits,ImageDecoder};
use std::io::{Cursor,Read,Write};
use tiny_skia::Pixmap;
use serde_json::{json,Value};
pub const RASTER_ATTRIBUTE:&str="openaec::paint::rasterImage";
pub const IMAGE_EXTENSIONS:&[&str]=&["png","jpg","jpeg","jpe","jfif","webp","gif","bmp","dib","tif","tiff","svg","svgz","ico","tga","pnm","ppm","pgm","pbm","pam","qoi","hdr","exr","dds","ff","avif","heic","heif","psd","psb","jp2","j2k","jpx","jxl","dng","raw","cr2","cr3","nef","nrw","arw","orf","rw2","pef","raf","srw","sr2","kdc","dcr"];
pub fn decode_png(bytes:&[u8])->Result<Pixmap>{ensure!(bytes.len()<=MAX_BYTES,"Image exceeds 64 MB");let reader=ImageReader::with_format(Cursor::new(bytes),ImageFormat::Png);let (w,h)=ImageReader::with_format(Cursor::new(bytes),ImageFormat::Png).into_dimensions()?;validate_dimensions(w,h)?;let image=reader.decode()?;pixmap_from_rgba(image.width(),image.height(),image.to_rgba8().as_raw())}
pub fn image_operation(pixmap:&Pixmap)->Result<Operation>{Ok(Operation{kind:Kind::Image,data:Some(format!("data:image/png;base64,{}",STANDARD.encode(pixmap.encode_png()?))),source_size:Some(Size{width:pixmap.width(),height:pixmap.height()}),..Default::default()})}
pub fn import_image(bytes:&[u8],name:&str)->Result<Document>{
    ensure!(bytes.len()<=MAX_BYTES,"Image exceeds 64 MB");
    if name.to_ascii_lowercase().ends_with(".ifcx")||name.to_ascii_lowercase().ends_with(".ifcxb"){return read_ifcx(bytes);}
    let ext=name.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    let decoded=match ext.as_str(){
     "svg"|"svgz"=>decode_svg(bytes)?,
     "heic"|"heif"=>{let options=heic_rs::DecodeOptions{layout:heic_rs::PixelLayout::Rgba8,max_pixels:Some(64_000_000),..Default::default()};let p=heic_rs::decode(bytes,&options).map_err(|e|anyhow::anyhow!(e.to_string()))?;let img=image::RgbaImage::from_raw(p.width,p.height,p.data).ok_or_else(||anyhow::anyhow!("Invalid HEIC pixels"))?;fit_image(DynamicImage::ImageRgba8(img))?},
     "psd"|"psb"=>{let p=psd::Psd::from_bytes(bytes)?;ensure!(u64::from(p.width())*u64::from(p.height())<=64_000_000,"Source exceeds 64 megapixels");let img=image::RgbaImage::from_raw(p.width(),p.height(),p.rgba()).ok_or_else(||anyhow::anyhow!("Invalid PSD pixels"))?;fit_image(DynamicImage::ImageRgba8(img))?},
     "jxl"=>{let mut decoder=jxl_oxide::integration::JxlDecoder::new(Cursor::new(bytes))?;decode_bounded(&mut decoder)?;fit_image(DynamicImage::from_decoder(decoder)?)?},
     "jp2"|"j2k"|"jpx"=>{let mut decoder=hayro_jpeg2000::integration::Jp2Decoder::new(Cursor::new(bytes))?;decode_bounded(&mut decoder)?;fit_image(DynamicImage::from_decoder(decoder)?)?},
     "avif"=>decode_avif(bytes)?,
     "raw"|"dng"|"cr2"|"cr3"|"nef"|"nrw"|"arw"|"orf"|"rw2"|"pef"|"raf"|"srw"|"sr2"|"kdc"|"dcr"=>{let source=rawler::rawsource::RawSource::new_from_slice(bytes);let decoder=rawler::get_decoder(&source)?;let params=rawler::decoders::RawDecodeParams::default();let info=decoder.raw_image(&source,&params,true)?;ensure!(info.width as u64*info.height as u64<=64_000_000,"Source exceeds 64 megapixels");let raw=decoder.raw_image(&source,&params,false)?;let img=rawler::imgop::develop::RawDevelop::default().develop_intermediate(&raw)?.to_dynamic_image().ok_or_else(||anyhow::anyhow!("Could not develop RAW"))?;fit_image(img)?},
     _=>{let mut reader=ImageReader::new(Cursor::new(bytes)).with_guessed_format()?;let mut limits=Limits::default();limits.max_image_width=Some(32768);limits.max_image_height=Some(32768);limits.max_alloc=Some(320_000_000);reader.limits(limits);let mut decoder=reader.into_decoder()?;decode_bounded(&mut decoder)?;let orientation=decoder.orientation()?;let mut img=DynamicImage::from_decoder(decoder)?;img.apply_orientation(orientation);fit_image(img)?}
    };
    let title=name.rsplit(['/', '\\']).next().unwrap_or(name).rsplit_once('.').map(|(base,_)|base).unwrap_or(name);
    let mut doc=Document::new(decoded.width(),decoded.height(),title.chars().take(120).collect::<String>());doc.background="transparent".into();doc.operations.push(image_operation(&decoded)?);doc.validate()?;Ok(doc)
}
fn fit_image(img:DynamicImage)->Result<Pixmap>{let scale=(MAX_SIDE as f64/img.width() as f64).min(MAX_SIDE as f64/img.height() as f64).min((MAX_PIXELS as f64/(u64::from(img.width())*u64::from(img.height())) as f64).sqrt()).min(1.);let img=if scale<1.{img.resize((img.width() as f64*scale).floor().max(1.) as u32,(img.height() as f64*scale).floor().max(1.) as u32,image::imageops::FilterType::Lanczos3)}else{img};pixmap_from_rgba(img.width(),img.height(),img.to_rgba8().as_raw())}
fn decode_svg(bytes:&[u8])->Result<Pixmap>{
    let mut options=resvg::usvg::Options::default();options.image_href_resolver.resolve_string=Box::new(|_,_|None);options.fontdb_mut().load_font_data(include_bytes!("../../../assets/fonts/Inter-Regular.ttf").to_vec());options.font_family="Inter".into();
    let tree=resvg::usvg::Tree::from_data(bytes,&options)?;let size=tree.size();let scale=(MAX_SIDE as f32/size.width()).min(MAX_SIDE as f32/size.height()).min((MAX_PIXELS as f32/(size.width()*size.height())).sqrt()).min(1.);let w=(size.width()*scale).round().max(1.) as u32;let h=(size.height()*scale).round().max(1.) as u32;validate_dimensions(w,h)?;
    let mut image=Pixmap::new(w,h).ok_or_else(||anyhow::anyhow!("SVG allocation failed"))?;resvg::render(&tree,tiny_skia::Transform::from_scale(scale,scale),&mut image.as_mut());Ok(image)
}
pub fn export_image(doc:&Document,extension:&str)->Result<Vec<u8>>{
    if extension=="ifcx"||extension=="ifcxb"{return write_ifcx(doc,extension=="ifcxb");}
    if extension=="svg"{return Ok(crate::edit::svg(doc)?.into_bytes());}
    let p=crate::render(doc)?;let mut output=Cursor::new(Vec::new());let rgba=image::RgbaImage::from_raw(doc.width,doc.height,rgba(&p)).unwrap();
    match extension {"jpg"|"jpeg"=>{let mut rgb=image::RgbImage::new(doc.width,doc.height);for (to,from) in rgb.pixels_mut().zip(rgba.pixels()){let a=from[3] as f32/255.;*to=image::Rgb(std::array::from_fn(|i|(from[i] as f32*a+255.*(1.-a)).round() as u8));}DynamicImage::ImageRgb8(rgb).write_to(&mut output,ImageFormat::Jpeg)?;},"webp"=>DynamicImage::ImageRgba8(rgba).write_to(&mut output,ImageFormat::WebP)?,"png"=>return crate::png(doc),_=>bail!("Unsupported export format")};Ok(output.into_inner())
}
pub fn write_ifcx(doc:&Document,binary:bool)->Result<Vec<u8>>{
    let uri=format!("data:image/png;base64,{}",STANDARD.encode(crate::png(doc)?));let value=json!({"header":{"id":uuid::Uuid::new_v4(),"ifcxVersion":"ifcx_alpha","dataVersion":"1.0.0","author":format!("OpenAEC Paint Studio {VERSION}")},"imports":[],"schemas":{RASTER_ATTRIBUTE:{"value":{"dataType":"Object"}}},"data":[{"path":"/paint/image","attributes":{RASTER_ATTRIBUTE:{"version":1,"name":doc.name,"width":doc.width,"height":doc.height,"mimeType":"image/png","uri":uri}}}]});
    if !binary{return Ok(serde_json::to_vec_pretty(&value)?);}let mut body=vec![0xd9,0xd9,0xf7];ciborium::into_writer(&value,&mut body)?;ensure!(body.len()<=MAX_BYTES,"IFCX exceeds 64 MB");let mut gz=flate2::write::GzEncoder::new(Vec::new(),flate2::Compression::default());gz.write_all(&body)?;Ok(gz.finish()?)
}
pub fn read_ifcx(input:&[u8])->Result<Document>{
    ensure!(input.len()<=MAX_BYTES,"IFCX exceeds 64 MB");let expanded;if input.starts_with(&[0x1f,0x8b]){let mut data=Vec::new();flate2::read::GzDecoder::new(input).take((MAX_BYTES+1) as u64).read_to_end(&mut data)?;ensure!(data.len()<=MAX_BYTES,"IFCX exceeds 64 MB");expanded=data;}else{expanded=input.to_vec();}
    let value:Value=if expanded.starts_with(&[0xd9,0xd9,0xf7]){ciborium::from_reader(&expanded[3..])?}else{serde_json::from_slice(&expanded)?};ensure!(value["header"]["ifcxVersion"]=="ifcx_alpha","Unsupported IFCX");let data=value["data"].as_array().ok_or_else(||anyhow::anyhow!("IFCX data is missing"))?;ensure!(data.len()<=20000,"Too many IFCX nodes");let matches:Vec<_>=data.iter().filter_map(|n|n["attributes"].get(RASTER_ATTRIBUTE)).collect();ensure!(matches.len()==1,"IFCX must contain exactly one Paint raster image");let r=matches[0];ensure!(r["version"]==1&&r["mimeType"]=="image/png","Unsupported IFCX image");let width=r["width"].as_u64().unwrap_or(0);let height=r["height"].as_u64().unwrap_or(0);ensure!(width<=4096&&height<=4096,"Invalid IFCX dimensions");validate_dimensions(width as u32,height as u32)?;
    let uri=r["uri"].as_str().ok_or_else(||anyhow::anyhow!("Missing PNG"))?;let base=uri.strip_prefix("data:image/png;base64,").ok_or_else(||anyhow::anyhow!("IFCX requires embedded PNG"))?;ensure!(uri.len()<=48_000_000,"Embedded PNG too large");let bytes=STANDARD.decode(base)?;let p=decode_png(&bytes)?;ensure!(p.width()==width as u32&&p.height()==height as u32,"IFCX and PNG dimensions differ");let name=r["name"].as_str().unwrap_or("Untitled");let mut doc=Document::new(p.width(),p.height(),name);doc.background="transparent".into();doc.operations.push(image_operation(&p)?);doc.validate()?;Ok(doc)
}

fn decode_bounded(decoder:&mut impl ImageDecoder)->Result<()>{let (w,h)=decoder.dimensions();ensure!(u64::from(w)*u64::from(h)<=64_000_000,"Source exceeds 64 megapixels");let mut limits=Limits::default();limits.max_alloc=Some(320_000_000);decoder.set_limits(limits)?;Ok(())}
#[cfg(not(target_arch="wasm32"))]
fn decode_avif(bytes:&[u8])->Result<Pixmap>{
 // Walk bounded container boxes and reject declared oversized planes before AV1 allocation.
 fn boxes(bytes:&[u8],depth:u8)->Result<()>{ensure!(depth<12,"AVIF nesting limit");let mut at=0;while at+8<=bytes.len(){let size=u32::from_be_bytes(bytes[at..at+4].try_into().unwrap()) as usize;let kind=&bytes[at+4..at+8];if size<8||at+size>bytes.len(){break;}let body=&bytes[at+8..at+size];if kind==b"ispe"&&body.len()>=12{let w=u32::from_be_bytes(body[4..8].try_into().unwrap());let h=u32::from_be_bytes(body[8..12].try_into().unwrap());ensure!(w as u64*h as u64<=64_000_000,"AVIF source exceeds 64 megapixels");}if matches!(kind,b"meta"|b"iprp"|b"ipco"){let skip=if kind==b"meta"{4}else{0};if body.len()>skip{boxes(&body[skip..],depth+1)?;}}at+=size;}Ok(())}boxes(bytes,0)?;
 let decoded=avif_decode::Decoder::from_avif(bytes)?.to_image()?;
 macro_rules! convert {($p:expr,$variant:ident)=>{{let p=$p;let w=p.width() as u32;let h=p.height() as u32;let data=bytemuck::cast_slice(p.buf()).to_vec();DynamicImage::$variant(image::ImageBuffer::from_raw(w,h,data).ok_or_else(||anyhow::anyhow!("Invalid AVIF pixels"))?)}}}
 let img=match decoded{avif_decode::Image::Rgb8(p)=>convert!(p,ImageRgb8),avif_decode::Image::Rgba8(p)=>convert!(p,ImageRgba8),avif_decode::Image::Gray8(p)=>convert!(p,ImageLuma8),avif_decode::Image::Rgb16(p)=>convert!(p,ImageRgb16),avif_decode::Image::Rgba16(p)=>convert!(p,ImageRgba16),avif_decode::Image::Gray16(p)=>convert!(p,ImageLuma16)};fit_image(img)
}

#[cfg(target_arch="wasm32")]fn decode_avif(_:&[u8])->Result<Pixmap>{bail!("The Rust browser adapter decodes AVIF before importing its PNG raster")}
