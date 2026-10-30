//! Paint control geometry, kept separate from Blueprint's B2 icon collection.
macro_rules! svg {($path:literal)=>{concat!("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.7\" stroke-linecap=\"round\" stroke-linejoin=\"round\">",$path,"</svg>")}}
pub const OPEN:&str=svg!("<path d=\"M3 7h7l2-3h8v5M3 8h18l-3 12H3z\"/>");
pub const SAVE:&str=svg!("<path d=\"M4 3h13l4 4v14H3V3zM7 3v6h10V3M7 21v-8h10v8\"/>");
pub const NEW:&str=svg!("<path d=\"M5 2h9l5 5v15H5zM14 2v6h5M8 14h8M12 10v8\"/>");
pub const EXPORT:&str=svg!("<path d=\"M12 3v13m-5-5 5 5 5-5M4 17v4h16v-4\"/>");
pub const BRUSH:&str=svg!("<path d=\"m9 14 10-11 2 2-10 11M9 14c-5-2-2 7-7 6 6 5 13-1 9-4z\"/>");
pub const ERASER:&str=svg!("<path d=\"m3 14 10-11 8 8-10 10H8zM8 9l8 8M12 21h10\"/>");
pub const FILL:&str=svg!("<path d=\"m3 11 8-8 10 10-8 8zM3 11h16M8 2l8 8M21 17c-4 5 3 5 0 0\"/>");
pub const PICKER:&str=svg!("<path d=\"m4 16 10-10 4 4L8 20H4zM13 4l7 7M17 5l3-3 2 2-3 3\"/>");
pub const LINE:&str=svg!("<path d=\"m4 20 16-16\"/>");
pub const RECT:&str=svg!("<rect x=\"4\" y=\"5\" width=\"16\" height=\"14\" rx=\"1\"/>");
pub const ELLIPSE:&str=svg!("<ellipse cx=\"12\" cy=\"12\" rx=\"9\" ry=\"7\"/>");
pub const TEXT:&str=svg!("<path d=\"M3 6V3h18v3M12 3v18m-4 0h8\"/>");
pub const SELECT:&str=svg!("<path d=\"m5 3 14 10-7 1-3 7z\"/>");
pub const CROP:&str=svg!("<path d=\"M7 2v15h15M2 7h15v15M4 20 20 4\"/>");
pub const IMAGE:&str=svg!("<rect x=\"3\" y=\"3\" width=\"18\" height=\"18\" rx=\"2\"/><circle cx=\"8\" cy=\"8\" r=\"2\"/><path d=\"m3 17 5-5 4 4 4-7 5 8\"/>");
pub const VIEW:&str=svg!("<path d=\"M2 12C7 1 17 1 22 12 17 23 7 23 2 12z\"/><circle cx=\"12\" cy=\"12\" r=\"3\"/>");
pub const COLOR:&str=svg!("<circle cx=\"12\" cy=\"12\" r=\"9\"/><path d=\"M12 3v18M3 12h18\"/>");
pub const SETTINGS:&str=svg!("<path d=\"M4 6h16M4 12h16M4 18h16\"/><circle cx=\"9\" cy=\"6\" r=\"2\"/><circle cx=\"16\" cy=\"12\" r=\"2\"/><circle cx=\"8\" cy=\"18\" r=\"2\"/>");
pub const UNDO:&str=svg!("<path d=\"m7 4-5 5 5 5M2 9h12c10 0 10 12 0 12\"/>");
pub const REDO:&str=svg!("<path d=\"m17 4 5 5-5 5m5-5H10C0 9 0 21 10 21\"/>");
pub const FIT:&str=svg!("<path d=\"M3 9V3h6m6 0h6v6M3 15v6h6m6 0h6v-6M8 8h8v8H8z\"/>");
pub const ROTATE:&str=svg!("<path d=\"M4 8a9 9 0 1 1-1 7M4 3v6h6\"/>");
pub const FLIP:&str=svg!("<path d=\"M12 2v20M3 6v12l6-6zm18 0v12l-6-6z\"/>");
pub const CLOSE:&str=svg!("<path d=\"m5 5 14 14M5 19 19 5\"/>");
pub const CHECK:&str=svg!("<path d=\"m4 12 5 5L20 5\"/>");
