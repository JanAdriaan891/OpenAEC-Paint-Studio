export type Adjustments = { brightness:number; contrast:number; saturation:number };
export function adjustPixels(data:Uint8ClampedArray,settings:Adjustments,kind:'adjust'|'grayscale'|'invert'|'sepia'='adjust') {
  const contrast=Math.max(-100,Math.min(100,settings.contrast))*2.55;
  const factor=(259*(contrast+255))/(255*(259-contrast));
  for(let i=0;i<data.length;i+=4) {
    let r=data[i],g=data[i+1],b=data[i+2];
    if(kind==='invert'){r=255-r;g=255-g;b=255-b;}
    else if(kind==='sepia'){const a=r,c=g,d=b;r=.393*a+.769*c+.189*d;g=.349*a+.686*c+.168*d;b=.272*a+.534*c+.131*d;}
    else {const luminance=.2126*r+.7152*g+.0722*b,saturation=kind==='grayscale'?0:settings.saturation/100;r=luminance+(r-luminance)*saturation;g=luminance+(g-luminance)*saturation;b=luminance+(b-luminance)*saturation;}
    for(const [j,v] of [r,g,b].entries())data[i+j]=Math.max(0,Math.min(255,factor*(v-128)+128+settings.brightness*2.55));
  }
}
