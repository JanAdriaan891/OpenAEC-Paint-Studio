//! Native OpenAEC Paint Studio and its browser build, both authored in Rust.
mod canvas;
mod icons;
mod platform;
mod view;
#[cfg(target_arch="wasm32")]mod web;
use iced::{Element,Task,Subscription,Size,Point,Vector,widget::{image,text_editor}};
use openaec_ui::{design::{Look,ThemeId},fonts};
use paint_core::{*,edit::{self,Editor,Tool,TransformAction,Filter}};
use canvas::Input;
pub const VERSION:&str=env!("CARGO_PKG_VERSION");
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub enum Dialog{Settings,New,Resize,Adjust,Api,Close,Replace}
#[derive(Debug,Clone,Copy)]pub enum Command{Open,OpenIfcx,New,Save,Export(&'static str),Undo,Redo,SelectAll,Delete,Fit,Actual,Zoom(f32),Grid,Focus,Inspector,Settings,Resize,Transform(TransformAction),Filter(Filter),Adjust,ApplyStyle,ApplyText,Cancel,Api,Quit}
#[derive(Debug,Clone)]pub enum Msg{DraftLoaded(Result<Option<Document>,String>),DraftStored(u64,Result<(),String>),Modifiers(iced::keyboard::Modifiers),Command(Command),Tool(Tool),Color(String),Size(String),Opacity(f32),Filled(bool),Bold(bool),Font(String),Vectors(bool),Theme(ThemeId),English(bool),Menu(Option<usize>),Dialog(Option<Dialog>),Width(String),Height(String),Name(String),Apply,Text(text_editor::Action),Canvas(Input,Size),Resized(Size),Key(iced::keyboard::Key,iced::keyboard::Modifiers),Opened(platform::OpenResult),Saved(Result<bool,String>,bool),Launch(std::path::PathBuf),CloseWindow,Exit,Replace,ApiBase(String),ApiToken(String),ApiAction(&'static str),ApiResult(Result<serde_json::Value,String>,&'static str),ApiId(String),Correction(u8,f32)}
#[derive(Debug)]pub struct TextDraft{pub content:text_editor::Content,pub at:paint_core::Point,pub index:Option<usize>}
#[derive(Debug,Clone)]pub enum Gesture{Draw(Operation),Selection{original:Document,at:paint_core::Point,bounds:Bounds,resize:bool},Pan{at:Point,original:Vector}}
pub struct App{
 pub pending_draft:Option<Document>,pub draft_generation:u64,pub modifiers:iced::keyboard::Modifiers,pub editor:Editor,pub tool:Tool,pub style:Operation,pub image:image::Handle,pub preview:Option<Document>,pub gesture:Option<Gesture>,pub text_box:Option<TextDraft>,pub settings:platform::Settings,pub menu:Option<usize>,pub dialog:Option<Dialog>,pub pending_document:Option<Document>,pub width:Size,pub canvas_size:Size,pub zoom:f32,pub pan:Vector,pub auto_fit:bool,pub grid:bool,pub focus:bool,pub inspector:bool,pub status:String,pub busy:bool,pub color_text:String,pub size_text:String,pub width_text:String,pub height_text:String,pub name_text:String,pub api_base:String,pub api_token:String,pub api_id:String,pub api_revision:Option<u64>,pub api_documents:Vec<serde_json::Value>,pub corrections:[f32;3],pub closing_after_save:bool,
}
impl Default for App{fn default()->Self{Self::with_document(platform::draft().unwrap_or_default())}}
impl App{
 pub fn with_document(document:Document)->Self{
  let settings=platform::settings();let w=document.width.to_string();let h=document.height.to_string();let image=Self::handle(&document).unwrap_or_else(|_|image::Handle::from_rgba(1,1,vec![255,255,255,255]));
  let mut app=Self{pending_draft:None,draft_generation:0,modifiers:iced::keyboard::Modifiers::default(),editor:Editor::new(document),tool:Tool::Brush,style:Operation::default(),image,preview:None,gesture:None,text_box:None,settings,menu:None,dialog:None,pending_document:None,width:Size::new(1440.,940.),canvas_size:Size::new(1200.,700.),zoom:75.,pan:Vector::ZERO,auto_fit:true,grid:false,focus:false,inspector:true,status:"Gereed · Rust".into(),busy:false,color_text:"#36363e".into(),size_text:"8".into(),width_text:w,height_text:h,name_text:"Untitled".into(),api_base:"http://127.0.0.1:47831".into(),api_token:String::new(),api_id:String::new(),api_revision:None,api_documents:vec![],corrections:[0.,0.,100.],closing_after_save:false};app.fit();app
 }
 pub fn look(&self)->Look{self.settings.theme.into()}
 pub fn tr<'a>(&self,nl:&'a str,en:&'a str)->&'a str{if self.settings.english{en}else{nl}}
 fn handle(doc:&Document)->anyhow::Result<image::Handle>{let p=paint_core::render(doc)?;Ok(image::Handle::from_rgba(p.width(),p.height(),paint_core::render::rgba(&p)))}
 fn repaint(&mut self){match Self::handle(self.preview.as_ref().unwrap_or(&self.editor.document)){Ok(handle)=>self.image=handle,Err(error)=>self.status=error.to_string()}}
 fn persist(&mut self){self.pending_draft=Some(self.editor.document.clone());}
 pub fn update(&mut self,message:Msg)->Task<Msg>{let task=self.update_inner(message);if let Some(doc)=self.pending_draft.take(){self.draft_generation+=1;let generation=self.draft_generation;Task::batch([task,Task::perform(platform::store_draft(doc,generation),move|r|Msg::DraftStored(generation,r))])}else{task}}
 fn changed(&mut self,result:anyhow::Result<()>){if let Err(e)=result{self.status=e.to_string();}else{self.persist();self.status=self.tr("Concept opslaan…","Saving draft…").into();}self.repaint();}
 fn commit(&mut self,doc:Document){let result=self.editor.commit(doc);self.changed(result);}
 pub fn origin(&self,size:Size)->Point{Point::new((size.width-self.editor.document.width as f32*self.zoom/100.)/2.+self.pan.x,(size.height-self.editor.document.height as f32*self.zoom/100.)/2.+self.pan.y)}
 fn location(&self,at:Point)->paint_core::Point{let o=self.origin(self.canvas_size);paint_core::Point::new(((at.x-o.x)*100./self.zoom).clamp(-8192.,8192.),((at.y-o.y)*100./self.zoom).clamp(-8192.,8192.))}
 fn fit(&mut self){let sidebar=if self.inspector&&self.width.width>=1000.&&!self.focus{230.}else{0.};let h=if self.focus{self.width.height-90.}else{self.width.height-238.};self.canvas_size=Size::new(self.width.width-sidebar,h);self.zoom=(((self.canvas_size.width-64.)/self.editor.document.width as f32).min((h-64.)/self.editor.document.height as f32)*100.).floor().clamp(10.,400.);self.pan=Vector::ZERO;self.auto_fit=true;}
 fn zoom_at(&mut self,value:f32,at:Point){let old=self.location(at);self.zoom=value.clamp(10.,400.);self.pan=Vector::new(at.x-old.x*self.zoom/100.-(self.canvas_size.width-self.editor.document.width as f32*self.zoom/100.)/2.,at.y-old.y*self.zoom/100.-(self.canvas_size.height-self.editor.document.height as f32*self.zoom/100.)/2.);self.auto_fit=false;}
 pub fn selection_bounds(&self)->Option<Bounds>{if self.tool!=Tool::Select{return None;}let doc=self.preview.as_ref().unwrap_or(&self.editor.document);if self.editor.selection.is_empty(){return None;}Some(self.editor.selection.iter().filter_map(|i|doc.operations.get(*i)?.bounds(doc.size())).reduce(Bounds::union).unwrap_or(Bounds{x:0.,y:0.,width:doc.width as f32,height:doc.height as f32}))}
 pub fn crop_bounds(&self)->Option<Bounds>{if self.tool!=Tool::Crop{return None;}if let Some(Gesture::Draw(op))=&self.gesture{let a=op.from?;let b=op.to?;Some(Bounds{x:a.x.min(b.x),y:a.y.min(b.y),width:(a.x-b.x).abs(),height:(a.y-b.y).abs()})}else{None}}
 fn cancel(&mut self){self.gesture=None;self.preview=None;self.text_box=None;self.menu=None;self.dialog=None;self.pending_document=None;self.tool=Tool::Select;self.repaint();}
 fn load(&mut self,doc:Document){self.editor=Editor::new(doc);self.preview=None;self.gesture=None;self.text_box=None;self.api_id.clear();self.api_revision=None;self.fit();self.persist();self.repaint();self.status=self.tr("Afbeelding geopend","Image opened").into();}
 fn apply_text(&mut self){let Some(draft)=self.text_box.take() else{return};let text=draft.content.text().trim_end_matches('\n').to_owned();if text.trim().is_empty(){return;}let mut op=draft.index.and_then(|i|self.editor.document.operations.get(i).cloned()).unwrap_or_else(||Operation{kind:Kind::Text,at:Some(draft.at),..self.style.clone()});op.text=Some(text);op.color=self.style.color.clone();op.size=self.style.size;op.opacity=self.style.opacity;op.bold=self.style.bold;op.font_family=self.style.font_family.clone();let result=if let Some(i)=draft.index{self.editor.selection=vec![i];self.editor.replace_selected(op)}else{self.editor.add(op)};self.changed(result);}
 fn update_inner(&mut self,message:Msg)->Task<Msg>{
  match message{
   Msg::DraftLoaded(result)=>{match result{Ok(Some(doc)) if !self.editor.dirty=>self.load(doc),Err(e)=>self.status=format!("Concept kon niet worden hersteld: {e}"),_=>{}}},
   Msg::DraftStored(generation,result)=>{if generation==self.draft_generation{self.status=match result{Ok(())=>self.tr("Concept opgeslagen · Rust","Draft saved · Rust").into(),Err(e)=>format!("Concept kon niet worden opgeslagen: {e}")};}},
   Msg::Modifiers(m)=>self.modifiers=m,
   Msg::Command(c)=>return self.command(c),
   Msg::Tool(tool)=>{self.gesture=None;self.preview=None;self.tool=tool;self.menu=None;if tool==Tool::Text&&self.style.size<24.{self.style.size=30.;self.size_text="30".into();}self.repaint();},
   Msg::Color(value)=>{self.color_text=value.clone();if parse_color(&value).is_ok(){self.style.color=value;}},
   Msg::Size(value)=>{if let Ok(v)=value.parse::<f32>(){if (1. ..=200.).contains(&v){self.style.size=v;}}self.size_text=value;},
   Msg::Opacity(v)=>self.style.opacity=v/100.,Msg::Filled(v)=>self.style.filled=v,Msg::Bold(v)=>self.style.bold=v,Msg::Font(v)=>self.style.font_family=Some(v),
   Msg::Vectors(v)=>{let mut doc=self.editor.document.clone();doc.retain_vectors=v;self.commit(doc);},
   Msg::Theme(t)=>{self.settings.theme=t;platform::store_settings(&self.settings);},Msg::English(v)=>{self.settings.english=v;platform::store_settings(&self.settings);},
   Msg::Menu(m)=>self.menu=m,Msg::Dialog(d)=>{self.dialog=d;self.menu=None;},Msg::Width(v)=>self.width_text=v,Msg::Height(v)=>self.height_text=v,Msg::Name(v)=>self.name_text=v,
   Msg::Text(action)=>{if let Some(d)=&mut self.text_box{d.content.perform(action);}},
   Msg::Canvas(input,size)=>{if size.width>0.&&size.height>0.{self.canvas_size=size;}self.canvas(input);},
   Msg::Resized(size)=>{self.width=size;if self.auto_fit{self.fit();}},
   Msg::Key(key,modifiers)=>return self.key(key,modifiers),
   Msg::Opened(result)=>{self.busy=false;match result{Ok(Some((name,bytes)))=>match paint_core::codec::import_image(&bytes,&name){Ok(doc)=>{if self.editor.dirty{self.pending_document=Some(doc);self.dialog=Some(Dialog::Replace);}else{self.load(doc);}},Err(e)=>self.status=e.to_string()},Ok(None)=>{},Err(e)=>self.status=e}},
   Msg::Saved(result,save)=>{self.busy=false;match result{Ok(true)=>{if save{self.editor.dirty=false;}self.status=self.tr("Bestand opgeslagen","File saved").into();if self.closing_after_save{return self.update(Msg::Exit);}},Ok(false)=>self.closing_after_save=false,Err(e)=>{self.status=e;self.closing_after_save=false;}}},
   Msg::Launch(path)=>{#[cfg(not(target_arch="wasm32"))]{let result=(||{if std::fs::metadata(&path)?.len()>MAX_BYTES as u64{anyhow::bail!("Image exceeds 64 MB");}Ok(Some((path.file_name().unwrap_or_default().to_string_lossy().into_owned(),std::fs::read(&path)?)))})().map_err(|e:anyhow::Error|e.to_string());return self.update(Msg::Opened(result));}#[cfg(target_arch="wasm32")]{let _=path;}},
   Msg::Apply=>{let size=(self.width_text.parse::<u32>(),self.height_text.parse::<u32>());if let (Ok(w),Ok(h))=size{match self.dialog{Some(Dialog::New)=>{let doc=Document::new(w,h,self.name_text.clone());if let Err(e)=doc.validate(){self.status=e.to_string();}else{if self.editor.dirty{self.pending_document=Some(doc);self.dialog=Some(Dialog::Replace);}else{self.dialog=None;self.load(doc);}}},Some(Dialog::Resize)=>{match edit::resize(&self.editor.document,w,h){Ok(doc)=>{self.commit(doc);self.dialog=None;self.fit();},Err(e)=>self.status=e.to_string()}},Some(Dialog::Adjust)=>{match edit::filter(&self.editor.document,Filter::Adjust{brightness:self.corrections[0],contrast:self.corrections[1],saturation:self.corrections[2]}){Ok(doc)=>{self.commit(doc);self.dialog=None;},Err(e)=>self.status=e.to_string()}},_=>{}}}else{self.status="Vul geldige afmetingen in".into();}},
   Msg::CloseWindow=>{if self.editor.dirty{self.dialog=Some(Dialog::Close);}else{return self.update(Msg::Exit);}},
   Msg::Exit=>{return iced::exit();},
   Msg::Replace=>{if let Some(doc)=self.pending_document.take(){self.dialog=None;self.load(doc);}},
   Msg::ApiBase(v)=>self.api_base=v,Msg::ApiToken(v)=>self.api_token=v,Msg::ApiId(v)=>self.api_id=v,
   Msg::ApiAction(action)=>{self.busy=true;return Task::perform(platform::api(self.api_base.clone(),self.api_token.clone(),self.api_id.clone(),self.api_revision,self.editor.document.clone(),action),move|r|Msg::ApiResult(r,action));},
   Msg::ApiResult(result,action)=>{self.busy=false;match result{Err(e)=>self.status=e,Ok(value)=>{if action=="list"{self.api_documents=value.as_array().cloned().unwrap_or_default();}else{if action=="pull"{if let Ok(doc)=serde_json::from_value::<Document>(value["document"].clone()){if doc.validate().is_ok(){self.commit(doc);self.fit();}}}self.api_id=value["id"].as_str().unwrap_or("").into();self.api_revision=value["revision"].as_u64();}self.status="API verbonden".into();}}},
   Msg::Correction(i,v)=>self.corrections[i as usize]=v,
  }Task::none()
 }
 fn command(&mut self,c:Command)->Task<Msg>{
  if self.busy&&!matches!(c,Command::Cancel){return Task::none();}self.menu=None;
  match c{
   Command::Open|Command::OpenIfcx=>{self.busy=true;return Task::perform(platform::open(matches!(c,Command::OpenIfcx)),Msg::Opened);},
   Command::Save|Command::Export(_)=>{let ext=if let Command::Export(ext)=c{ext}else{"ifcx"};self.closing_after_save=self.dialog==Some(Dialog::Close);self.busy=true;return Task::perform(platform::save(self.editor.document.clone(),ext),move|r|Msg::Saved(r,matches!(c,Command::Save)));},
   Command::Undo=>{self.editor.undo();self.preview=None;self.gesture=None;self.persist();self.repaint();},Command::Redo=>{self.editor.redo();self.preview=None;self.gesture=None;self.persist();self.repaint();},
   Command::SelectAll=>{self.tool=Tool::Select;self.text_box=None;self.preview=None;self.gesture=None;self.editor.select_all();self.repaint();},Command::Delete=>{let result=self.editor.delete_selection();self.changed(result);},
   Command::Fit=>self.fit(),Command::Actual=>{self.zoom_at(100.,Point::new(self.canvas_size.width/2.,self.canvas_size.height/2.));},Command::Zoom(v)=>self.zoom_at(self.zoom+v,Point::new(self.canvas_size.width/2.,self.canvas_size.height/2.)),
   Command::Grid=>self.grid=!self.grid,Command::Focus=>{self.focus=!self.focus;if self.auto_fit{self.fit();}},Command::Inspector=>{self.inspector=!self.inspector;if self.auto_fit{self.fit();}},
   Command::New=>{self.width_text=self.editor.document.width.to_string();self.height_text=self.editor.document.height.to_string();self.name_text="Untitled".into();self.dialog=Some(Dialog::New);},
   Command::Resize=>{self.width_text=self.editor.document.width.to_string();self.height_text=self.editor.document.height.to_string();self.dialog=Some(Dialog::Resize);},
   Command::Settings=>self.dialog=Some(Dialog::Settings),Command::Api=>self.dialog=Some(Dialog::Api),Command::Adjust=>self.dialog=Some(Dialog::Adjust),
   Command::Transform(action)=>match edit::transform_document(&self.editor.document,action){Ok(doc)=>{self.commit(doc);if self.auto_fit{self.fit();}},Err(e)=>self.status=e.to_string()},
   Command::Filter(f)=>match edit::filter(&self.editor.document,f){Ok(doc)=>self.commit(doc),Err(e)=>self.status=e.to_string()},
   Command::ApplyStyle=>{if self.editor.selection.len()==1{let mut op=self.editor.document.operations[self.editor.selection[0]].clone();op.color=self.style.color.clone();op.size=self.style.size;op.opacity=self.style.opacity;op.bold=self.style.bold;op.font_family=self.style.font_family.clone();op.filled=self.style.filled;let result=self.editor.replace_selected(op);self.changed(result);}},
   Command::ApplyText=>self.apply_text(),Command::Cancel=>self.cancel(),Command::Quit=>return self.update(Msg::CloseWindow),
  }Task::none()
 }
 fn key(&mut self,key:iced::keyboard::Key,m:iced::keyboard::Modifiers)->Task<Msg>{
  use iced::keyboard::{Key,key::Named};
  if key==Key::Named(Named::Escape){self.cancel();return Task::none();}
  if self.dialog.is_some(){return Task::none();}
  if self.text_box.is_some(){if key==Key::Named(Named::Enter)&&(m.control()||m.logo()){self.apply_text();}return Task::none();}
  let c=if m.control()||m.logo(){match key.as_ref(){Key::Character("a"|"A")=>Some(Command::SelectAll),Key::Character("s"|"S")=>Some(Command::Save),Key::Character("o"|"O")=>Some(Command::Open),Key::Character("z"|"Z")=>Some(if m.shift(){Command::Redo}else{Command::Undo}),Key::Character("y"|"Y")=>Some(Command::Redo),_=>None}}else{match key.as_ref(){Key::Named(Named::Delete)=>Some(Command::Delete),Key::Named(Named::F11)=>Some(Command::Focus),Key::Character("b"|"B")=>return self.update(Msg::Tool(Tool::Brush)),Key::Character("e"|"E")=>return self.update(Msg::Tool(Tool::Eraser)),Key::Character("f"|"F")=>return self.update(Msg::Tool(Tool::Fill)),Key::Character("i"|"I")=>return self.update(Msg::Tool(Tool::Picker)),Key::Character("l"|"L")=>return self.update(Msg::Tool(Tool::Line)),Key::Character("r"|"R")=>return self.update(Msg::Tool(Tool::Rectangle)),Key::Character("o"|"O")=>return self.update(Msg::Tool(Tool::Ellipse)),Key::Character("t"|"T")=>return self.update(Msg::Tool(Tool::Text)),Key::Character("v"|"V")=>return self.update(Msg::Tool(Tool::Select)),Key::Character("c"|"C")=>return self.update(Msg::Tool(Tool::Crop)),Key::Character("[")=>return self.update(Msg::Size((self.style.size-1.).max(1.).to_string())),Key::Character("]")=>return self.update(Msg::Size((self.style.size+1.).min(200.).to_string())),_=>None}};
  if let Some(c)=c{return self.command(c);}Task::none()
 }
 fn canvas(&mut self,input:Input){
  if self.busy||self.dialog.is_some()||self.text_box.is_some(){return;}
  match input{
   Input::Cancel=>{self.gesture=None;self.preview=None;self.repaint();},
   Input::Wheel(delta,at,ctrl)=>{if ctrl{self.zoom_at(self.zoom+if delta>0.{10.}else{-10.},at);}else{self.pan.y+=delta;self.auto_fit=false;}},
   Input::Down(at,true)=>{self.gesture=Some(Gesture::Pan{at,original:self.pan});self.auto_fit=false;},
   Input::Down(at,false)=>{
    let p=self.location(at);let doc=&self.editor.document;if p.x<0.||p.y<0.||p.x>=doc.width as f32||p.y>=doc.height as f32{return;}
    if self.tool==Tool::Select{
     let bounds=self.editor.selected_bounds();let resize=bounds.is_some_and(|b|((p.x-b.x-b.width).powi(2)+(p.y-b.y-b.height).powi(2)).sqrt()<12.*100./self.zoom);
     if !resize&&!bounds.is_some_and(|b|b.contains(p)){self.editor.selection=self.editor.hit(p).into_iter().collect();}
     if let Some(bounds)=self.editor.selected_bounds(){self.gesture=Some(Gesture::Selection{original:self.editor.document.clone(),at:p,bounds,resize});if self.editor.selection.len()==1{self.style=self.editor.document.operations[self.editor.selection[0]].clone();self.size_text=self.style.size.to_string();self.color_text=self.style.color.clone();}}return;
    }
    if self.tool==Tool::Picker{if let Ok(pixmap)=paint_core::render(&self.editor.document){if let Some(pixel)=pixmap.pixel(p.x as u32,p.y as u32){let c=pixel.demultiply();self.style.color=format!("#{:02x}{:02x}{:02x}",c.red(),c.green(),c.blue());self.color_text=self.style.color.clone();}}self.tool=Tool::Brush;return;}
    if self.tool==Tool::Text{let index=self.editor.hit(p).filter(|i|self.editor.document.operations[*i].kind==Kind::Text);let (at,text)=if let Some(i)=index{let op=&self.editor.document.operations[i];self.style=op.clone();self.size_text=op.size.to_string();self.color_text=op.color.clone();(op.at.unwrap(),op.text.clone().unwrap_or_default())}else{(p,String::new())};self.text_box=Some(TextDraft{content:text_editor::Content::with_text(&text),at,index});return;}
    let mut op=self.style.clone();op.kind=self.tool.kind().unwrap_or(Kind::Rectangle);op.transform=None;op.text=None;op.data=None;op.source_size=None;op.points.clear();op.from=None;op.to=None;op.at=None;
    match self.tool{Tool::Brush|Tool::Eraser=>op.points.push(p),Tool::Fill=>{op.at=Some(p);let result=self.editor.add(op);self.changed(result);return;},_=>{op.from=Some(p);op.to=Some(p);}}
    self.gesture=Some(Gesture::Draw(op));
   },
   Input::Move(at)=>{
    let p=self.location(at);let Some(gesture)=&mut self.gesture else{return};match gesture{
     Gesture::Pan{at:from,original}=>self.pan=*original+(at-*from),
     Gesture::Draw(op)=>{if matches!(op.kind,Kind::Brush|Kind::Eraser){if op.points.len()<20000{op.points.push(p);}}else{let mut to=p;if self.modifiers.shift()&&self.tool!=Tool::Crop{if let Some(from)=op.from{let dx=p.x-from.x;let dy=p.y-from.y;if op.kind==Kind::Line{let distance=dx.hypot(dy);let angle=(dy.atan2(dx)/std::f32::consts::FRAC_PI_4).round()*std::f32::consts::FRAC_PI_4;to=paint_core::Point::new(from.x+distance*angle.cos(),from.y+distance*angle.sin());}else{let size=dx.abs().max(dy.abs());to=paint_core::Point::new(from.x+size*dx.signum(),from.y+size*dy.signum());}}}op.to=Some(to);}if self.tool!=Tool::Crop{let mut doc=self.editor.document.clone();doc.operations.push(op.clone());self.preview=Some(doc);self.repaint();}},
     Gesture::Selection{original,at:from,bounds,resize}=>{let dx=p.x-from.x;let dy=p.y-from.y;let matrix=if *resize{let sx=((bounds.width+dx)/bounds.width).max(0.05);let sy=((bounds.height+dy)/bounds.height).max(0.05);[sx,0.,0.,sy,bounds.x*(1.-sx),bounds.y*(1.-sy)]}else{[1.,0.,0.,1.,dx,dy]};let mut doc=original.clone();for &i in &self.editor.selection{doc.operations[i]=original.operations[i].transformed(matrix,original.size());}self.preview=Some(doc);self.repaint();},
    }
   },
   Input::Up=>{let crop=self.crop_bounds();let gesture=self.gesture.take();match gesture{Some(Gesture::Draw(op))=>{if self.tool==Tool::Crop{if let Some(b)=crop{if b.width>1.&&b.height>1.{match edit::crop(&self.editor.document,b){Ok(doc)=>{self.commit(doc);self.fit();self.tool=Tool::Select;},Err(e)=>self.status=e.to_string()}}}}else{self.preview=None;let result=self.editor.add(op);self.changed(result);}},Some(Gesture::Selection{..})=>{if let Some(doc)=self.preview.take(){self.commit(doc);}},_=>{}}self.preview=None;self.repaint();},
  }
 }
 pub fn subscription(&self)->Subscription<Msg>{iced::event::listen_with(|event,status,_|match event{
  iced::Event::Window(iced::window::Event::Resized(size)|iced::window::Event::Opened{size,..})=>Some(Msg::Resized(size)),iced::Event::Window(iced::window::Event::CloseRequested)=>Some(Msg::CloseWindow),iced::Event::Window(iced::window::Event::FileDropped(path))=>Some(Msg::Launch(path)),iced::Event::Window(iced::window::Event::Unfocused)=>Some(Msg::Canvas(Input::Cancel,Size::ZERO)),
  iced::Event::Keyboard(iced::keyboard::Event::ModifiersChanged(m))=>Some(Msg::Modifiers(m)),
  iced::Event::Keyboard(iced::keyboard::Event::KeyPressed{key,modifiers,..}) if status==iced::event::Status::Ignored||key==iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape)||(key==iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter)&&(modifiers.control()||modifiers.logo()))=>Some(Msg::Key(key,modifiers)),_=>None})}
 pub fn view(&self)->Element<'_,Msg>{view::view(self)}
}
pub fn run()->iced::Result{
 fonts::install_fallback();let builder=iced::application(||{let app=App::default();#[cfg(not(target_arch="wasm32"))]let launch=std::env::args_os().skip(1).map(std::path::PathBuf::from).find(|p|p.is_file()).map(|p|Task::done(Msg::Launch(p))).unwrap_or(Task::none());#[cfg(target_arch="wasm32")]let launch=Task::perform(platform::load_web_draft(),Msg::DraftLoaded);(app,launch)},App::update,App::view)
 .title(|app:&App|format!("{} — OpenAEC Paint Studio {VERSION}",app.editor.document.name)).subscription(App::subscription).theme(|app:&App|app.settings.theme.iced()).default_font(fonts::BODY)
 .window(iced::window::Settings{size:Size::new(1440.,940.),min_size:Some(Size::new(560.,560.)),exit_on_close_request:false,..Default::default()});
 fonts::FILES.iter().fold(builder,|b,font|b.font(*font)).run()
}
#[cfg(target_arch="wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]pub fn web_start(){console_error_panic_hook::set_once();web::install();let _=run();}

#[cfg(test)]mod tests;
