import { ImageMagick, Magick, MagickFormat, MagickImageInfo, MagickReadSettings, ResourceLimits, initializeImageMagick } from '@imagemagick/magick-wasm';
let initialization: Promise<void> | undefined;
export function initializeDecoder(wasm: Uint8Array) {
  return initialization ??= initializeImageMagick(wasm).then(() => {
    ResourceLimits.memory = 512n * 1024n * 1024n;
    ResourceLimits.maxMemoryRequest = 256n * 1024n * 1024n;
    ResourceLimits.disk = 0n; ResourceLimits.time = 25n;
    ResourceLimits.width = 32768n; ResourceLimits.height = 32768n;
    ResourceLimits.listLength = 128n;
  });
}
export function decoderFormats() {
  const excluded=new Set(['PDF','PDFA','PS','PS2','PS3','EPS','EPS2','EPS3','EPSF','EPSI','AI','HTTP','HTTPS','FTP','URL','FILE','INLINE','XC','CANVAS','LABEL','CAPTION','TEXT']);
  return Magick.supportedFormats.filter(f=>f.supportsReading&&!excluded.has(f.format)).map(f=>({format:f.format,description:f.description}));
}
export function decodeImage(bytes: Uint8Array, hint?: string) {
  const settings = new MagickReadSettings({frameIndex:0, frameCount:1});
  if(hint?.toUpperCase()==='SVG'||new TextDecoder().decode(bytes.subarray(0,2048)).includes('<svg'))settings.format=MagickFormat.Msvg;
  let info;
  try { info=MagickImageInfo.create(bytes,settings); }
  catch(error) {
    const aliases:Record<string,string>={JPG:'JPEG',JPE:'JPEG',TIF:'TIFF',TGA:'TGA',PBM:'PBM',PGM:'PGM',PPM:'PPM',PNM:'PNM'};
    const name=(hint||'').toUpperCase(),format=Magick.supportedFormats.find(f=>f.supportsReading&&f.format===(aliases[name]||name))?.format;
    if(!format)throw error;settings.format=format;info=MagickImageInfo.create(bytes,settings);
  }
  if(info.width*info.height>64_000_000) throw new Error('Image exceeds 64 megapixels');
  return ImageMagick.read(bytes, settings, image => {
    image.autoOrient();
    const original = {width:image.width,height:image.height};
    const scale=Math.min(1,4096/image.width,4096/image.height,Math.sqrt(8_388_608/(image.width*image.height)));
    if(scale<1) image.resize(Math.max(1,Math.floor(image.width*scale)),Math.max(1,Math.floor(image.height*scale)));
    image.strip();
    return image.write(MagickFormat.Png, data=>({bytes:data.slice(),width:image.width,height:image.height,format:info.format,original}));
  });
}
