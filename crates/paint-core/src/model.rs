use anyhow::{bail, ensure, Result};
use serde::{Deserialize, Serialize};

pub const MAX_BYTES: usize = 64_000_000;
pub const MAX_PIXELS: u64 = 8_388_608;
pub const MAX_SIDE: u32 = 4096;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub type Matrix = [f32; 6];
pub const IDENTITY: Matrix = [1., 0., 0., 1., 0., 0.];

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct Point { pub x: f32, pub y: f32 }
impl Point { pub fn new(x: f32, y: f32) -> Self { Self { x, y } } }
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Size { pub width: u32, pub height: u32 }
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Bounds { pub x: f32, pub y: f32, pub width: f32, pub height: f32 }
impl Bounds {
    pub fn contains(self, p: Point) -> bool { p.x >= self.x && p.y >= self.y && p.x <= self.x+self.width && p.y <= self.y+self.height }
    pub fn union(self, b: Self) -> Self { let x=self.x.min(b.x); let y=self.y.min(b.y); Self {x,y,width:(self.x+self.width).max(b.x+b.width)-x,height:(self.y+self.height).max(b.y+b.height)-y} }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all="lowercase")]
pub enum Kind { #[default] Brush, Eraser, Fill, Line, Rectangle, Ellipse, Text, Image }
fn default_color() -> String { "#36363e".into() }
fn default_size() -> f32 { 8. }
fn default_opacity() -> f32 { 1. }
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all="camelCase")]
pub struct Operation {
    #[serde(rename="type")] pub kind: Kind,
    #[serde(default, skip_serializing_if="Vec::is_empty")] pub points: Vec<Point>,
    #[serde(skip_serializing_if="Option::is_none")] pub from: Option<Point>,
    #[serde(skip_serializing_if="Option::is_none")] pub to: Option<Point>,
    #[serde(skip_serializing_if="Option::is_none")] pub at: Option<Point>,
    #[serde(skip_serializing_if="Option::is_none")] pub text: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")] pub data: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")] pub source_size: Option<Size>,
    #[serde(default="default_color")] pub color: String,
    #[serde(default="default_size")] pub size: f32,
    #[serde(default="default_opacity")] pub opacity: f32,
    #[serde(default)] pub filled: bool,
    #[serde(default)] pub bold: bool,
    #[serde(skip_serializing_if="Option::is_none")] pub font_family: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")] pub transform: Option<Matrix>,
}
impl Default for Operation {
    fn default() -> Self { Self { kind: Kind::Brush, points:vec![], from:None,to:None,at:None,text:None,data:None,source_size:None,color:default_color(),size:8.,opacity:1.,filled:false,bold:false,font_family:None,transform:None } }
}
impl Operation {
    pub fn validate(&self) -> Result<()> {
        parse_color(&self.color)?;
        ensure!(self.size.is_finite() && (1. ..=200.).contains(&self.size), "Size must be 1–200");
        ensure!(self.opacity.is_finite() && (0. ..=1.).contains(&self.opacity), "Opacity must be 0–1");
        if let Some(m)=self.transform { ensure!(m.iter().all(|v|v.is_finite() && v.abs()<=1_000_000.), "Invalid transform"); }
        for p in self.points.iter().chain(self.from.iter()).chain(self.to.iter()).chain(self.at.iter()) { ensure!(p.x.is_finite()&&p.y.is_finite()&&p.x.abs()<=8192.&&p.y.abs()<=8192.,"Invalid point"); }
        match self.kind {
            Kind::Brush|Kind::Eraser => ensure!(!self.points.is_empty()&&self.points.len()<=20_000,"Stroke needs 1–20000 points"),
            Kind::Line|Kind::Rectangle|Kind::Ellipse => ensure!(self.from.is_some()&&self.to.is_some(),"Shape needs from/to"),
            Kind::Text => { ensure!(self.at.is_some(),"Text needs a position"); let text=self.text.as_deref().unwrap_or(""); ensure!(!text.is_empty()&&text.chars().count()<=2000,"Text needs 1–2000 characters"); ensure!(matches!(self.font_family.as_deref(),None|Some("sans-serif"|"serif"|"monospace")),"Invalid font"); }
            Kind::Fill => ensure!(self.at.is_some(),"Fill needs a position"),
            Kind::Image => { let data=self.data.as_deref().unwrap_or(""); ensure!(data.starts_with("data:image/png;base64,")&&data.len()<=48_000_000,"Image must be an embedded PNG"); if let Some(s)=self.source_size {validate_dimensions(s.width,s.height)?;} }
        }
        Ok(())
    }
    pub fn transformed(&self, matrix: Matrix, old: Size) -> Self {
        let mut op=self.clone();op.transform=Some(compose(matrix,self.transform.unwrap_or(IDENTITY)));
        if op.kind==Kind::Image && op.source_size.is_none(){op.source_size=Some(old);}op
    }
    pub fn bounds(&self, document: Size) -> Option<Bounds> {
        let mut points=match self.kind {
            Kind::Brush|Kind::Eraser => self.points.clone(),
            Kind::Fill => return None,
            Kind::Line|Kind::Rectangle|Kind::Ellipse => {let a=self.from?;let b=self.to?;vec![a,b,Point::new(a.x,b.y),Point::new(b.x,a.y)]},
            Kind::Text => {let a=self.at?;let t=self.text.as_deref()?;let w=t.lines().map(|s|s.chars().count()).max().unwrap_or(1) as f32*self.size*0.65;let h=t.lines().count().max(1) as f32*self.size*1.3;vec![a,Point::new(a.x+w,a.y),Point::new(a.x,a.y+h),Point::new(a.x+w,a.y+h)]},
            Kind::Image => {let s=self.source_size.unwrap_or(document);vec![Point::default(),Point::new(s.width as f32,0.),Point::new(0.,s.height as f32),Point::new(s.width as f32,s.height as f32)]},
        };
        for p in &mut points {*p=transform_point(*p,self.transform.unwrap_or(IDENTITY));}
        if points.is_empty(){return None;}
        let x=points.iter().map(|p|p.x).fold(f32::INFINITY,f32::min);let y=points.iter().map(|p|p.y).fold(f32::INFINITY,f32::min);
        Some(Bounds{x,y,width:(points.iter().map(|p|p.x).fold(f32::NEG_INFINITY,f32::max)-x).max(1.),height:(points.iter().map(|p|p.y).fold(f32::NEG_INFINITY,f32::max)-y).max(1.)})
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all="camelCase")]
pub struct Document { pub format:String, pub version:u32, pub name:String, pub width:u32, pub height:u32, #[serde(default="yes")] pub retain_vectors:bool, pub background:String, pub operations:Vec<Operation> }
fn yes()->bool{true}
impl Default for Document { fn default()->Self { Self::new(1200,800,"Untitled") } }
impl Document {
    pub fn new(width:u32,height:u32,name:impl Into<String>)->Self {Self{format:"openaec-paint".into(),version:1,name:name.into(),width,height,retain_vectors:true,background:"#ffffff".into(),operations:vec![]}}
    pub fn size(&self)->Size{Size{width:self.width,height:self.height}}
    pub fn validate(&self)->Result<()> {
        ensure!(self.format=="openaec-paint"&&self.version==1,"Unsupported document");validate_dimensions(self.width,self.height)?;
        ensure!(!self.name.trim().is_empty()&&self.name.chars().count()<=120,"Invalid document name");
        if self.background!="transparent"{parse_color(&self.background)?;}
        ensure!(self.operations.len()<=2000,"At most 2000 operations");for op in &self.operations{op.validate()?;}Ok(())
    }
    pub fn from_json(bytes:&[u8])->Result<Self>{ensure!(bytes.len()<=MAX_BYTES,"Document exceeds 64 MB");let doc:Self=serde_json::from_slice(bytes)?;doc.validate()?;Ok(doc)}
}
pub fn validate_dimensions(w:u32,h:u32)->Result<()>{ensure!(w>0&&h>0&&w<=MAX_SIDE&&h<=MAX_SIDE&&u64::from(w)*u64::from(h)<=MAX_PIXELS,"Canvas is limited to 4096 px per side and 8 megapixels");Ok(())}
pub fn parse_color(hex:&str)->Result<[u8;4]>{if hex.len()!=7||!hex.starts_with('#'){bail!("Color must be #RRGGBB");}let n=u32::from_str_radix(&hex[1..],16)?;Ok([(n>>16) as u8,(n>>8) as u8,n as u8,255])}
pub fn compose(a:Matrix,b:Matrix)->Matrix{[a[0]*b[0]+a[2]*b[1],a[1]*b[0]+a[3]*b[1],a[0]*b[2]+a[2]*b[3],a[1]*b[2]+a[3]*b[3],a[0]*b[4]+a[2]*b[5]+a[4],a[1]*b[4]+a[3]*b[5]+a[5]]}
pub fn transform_point(p:Point,m:Matrix)->Point{Point::new(m[0]*p.x+m[2]*p.y+m[4],m[1]*p.x+m[3]*p.y+m[5])}
