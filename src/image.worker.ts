import wasmUrl from '@imagemagick/magick-wasm/magick.wasm?url';
import { initializeDecoder, decodeImage, decoderFormats } from '../shared/image-decoder';
const scope=self as unknown as {onmessage:(event:MessageEvent)=>void;postMessage:(value:unknown,transfer?:Transferable[])=>void};
scope.onmessage=async ({data})=>{
  try {
    const response=await fetch(wasmUrl); if(!response.ok) throw new Error('Image decoder could not be loaded');
    await initializeDecoder(new Uint8Array(await response.arrayBuffer()));
    if(data.type==='formats') scope.postMessage({formats:decoderFormats()});
    else { const result=decodeImage(new Uint8Array(data.bytes),data.hint); scope.postMessage(result,[result.bytes.buffer as ArrayBuffer]); }
  } catch(error) {scope.postMessage({error:error instanceof Error?error.message:String(error)});}
};
