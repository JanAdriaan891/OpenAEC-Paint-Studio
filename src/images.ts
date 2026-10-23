// Keep the image picker focused on images, including decoder-supported formats
// whose MIME type is not consistently registered by every operating system.
export const imageFileAccept = ['image/*', ...'png jpg jpeg jpe jfif gif webp bmp dib tif tiff heic heif avif psd psb svg svgz ico cur tga jp2 j2k jpx jpf jxl exr hdr pic pnm pbm pgm ppm pam qoi pcx xcf raw dng cr2 cr3 nef nrw arw srf sr2 orf raf rw2 pef srw dcr kdc mrw erf x3f 3fr iiq fits fit fts'.split(' ').map(extension => `.${extension}`)].join(',');
export type DecodedImage = {bytes:Uint8Array;width:number;height:number;format:string;original:{width:number;height:number}};
function workerRequest<T>(message:unknown, transfer:Transferable[]=[]):Promise<T> {
  const worker=new Worker(new URL('./image.worker.ts',import.meta.url),{type:'module'});
  return new Promise((resolve,reject)=>{
    const timer=setTimeout(()=>{worker.terminate();reject(new Error('Image processing timed out. Try a smaller file.'));},40000);
    worker.onmessage=({data})=>{clearTimeout(timer);worker.terminate();if(data.error)reject(new Error(data.error));else resolve(data);};
    worker.onerror=event=>{clearTimeout(timer);worker.terminate();reject(new Error(event.message||'Image decoder failed'));};
    worker.postMessage(message,transfer);
  });
}
export async function decodeFile(file:File) {
  // Native decoders keep the everyday Paint workflow instant. The WASM worker
  // handles TIFF, PSD, HEIC, RAW and other formats unsupported by this browser.
  if(/\.(png|jpe?g|jfif|gif|webp|bmp|svg|avif|ico)$/i.test(file.name)) {
    const url=URL.createObjectURL(file),image=new Image();
    try {
      image.src=url;await image.decode();
      const original={width:image.naturalWidth,height:image.naturalHeight};
      if(original.width*original.height>64_000_000)throw new Error('Image exceeds 64 megapixels');
      const scale=Math.min(1,4096/original.width,4096/original.height,Math.sqrt(8_388_608/(original.width*original.height)));
      const canvas=document.createElement('canvas');canvas.width=Math.max(1,Math.floor(original.width*scale));canvas.height=Math.max(1,Math.floor(original.height*scale));
      canvas.getContext('2d')!.drawImage(image,0,0,canvas.width,canvas.height);
      const blob=await new Promise<Blob>((resolve,reject)=>canvas.toBlob(value=>value?resolve(value):reject(new Error('Image conversion failed')),'image/png'));
      return {bytes:new Uint8Array(await blob.arrayBuffer()),width:canvas.width,height:canvas.height,format:file.name.split('.').pop()!.toUpperCase(),original};
    } catch { /* Continue with the broader ImageMagick decoder. */ }
    finally {URL.revokeObjectURL(url);}
  }
  const bytes=await file.arrayBuffer();return workerRequest<DecodedImage>({type:'decode',bytes,hint:file.name.split('.').pop()},[bytes]);
}
export const supportedFormats=()=>workerRequest<{formats:{format:string;description:string}[]}>({type:'formats'});
export function bytesToDataUrl(bytes:Uint8Array) {return new Promise<string>((resolve,reject)=>{const reader=new FileReader();reader.onload=()=>resolve(String(reader.result));reader.onerror=()=>reject(reader.error);reader.readAsDataURL(new Blob([bytes as BlobPart],{type:'image/png'}));});}
