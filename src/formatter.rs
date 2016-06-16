// (c) 2016 Productize SPRL <joost@productize.be>

use std::io;

use Sexp;
use symbolic_expressions::Result;
use symbolic_expressions::Formatter;

// custom symbolic_expressions formatter that aims to be
// kicad compatible

struct Indent {
    newline_before:i64,
    closing_on_new_line:bool,
    newline_after:i64,
}

impl Default for Indent {
    fn default() -> Indent {
        Indent {
            newline_before:0,
            closing_on_new_line:false,
            newline_after:0,
        }
    }
}


impl Indent {
    
    fn before(&mut self) {
        self.newline_before = 1;
    }
    
    fn close_on_new_line(&mut self) {
        self.closing_on_new_line = true;
    }
    
    fn before_double(&mut self) {
        self.newline_before = 2;
    }

    fn newline_after_closing(&mut self) {
        self.newline_after = 1;
    }
}


pub struct KicadFormatter {
    indent:i64,
    stack:Vec<Option<(String,Option<Indent>)>>,
    ind:Vec<u8>,
    pts_xy_count:i64,
}

impl KicadFormatter {
    
    pub fn new(initial_indent_level:i64) -> KicadFormatter {
        KicadFormatter {
            indent:initial_indent_level,
            stack:vec![],
            ind:vec![b' ',b' '], // two spaces
            pts_xy_count:0,
        }
    }

    fn is(&self, what:&'static str) -> bool {
        for x in &self.stack {
            if let Some((ref name,_)) = *x {
                if name == what {
                    return true
                }
            }
        }
        false
    }
    
    fn parent_is(&self, what:&'static str) -> bool {
        if let Some(s) = self.stack.last() {
            if let Some((ref t,_)) = *s {
                return t == what
            }
        } 
        false
    }
    
    fn indent<W:io::Write>(&self, writer:&mut W, nls:i64) -> Result<()> {
        for _ in 0..nls {
            try!(writer.write_all(b"\n"));
        }
        for _ in 0..self.indent {
            try!(writer.write_all(&self.ind));
        }
        Ok(())
    }

    fn indent_plus<W:io::Write>(&mut self, writer:&mut W, nls:i64) -> Result<()> {
        self.indent+=1;
        let res = self.indent(writer, nls);
        self.indent-=1;
        res
    }

    fn want_indent_module(&self, ele:&str) -> Option<Indent> {
        //if !self.is("module") {
        //    return None
        //}
        let mut indent = Indent::default();
        indent.before();
        if self.parent_is("module") {
            match ele {
                "at" | "descr" | "fp_line" | "fp_poly" |
                "pad" | "path" | "fp_circle"
                    => return Some(indent),
                "model" | "fp_text" => {
                    indent.close_on_new_line();
                    return Some(indent)
                },
                _ => (),
            }
        } 
        if self.parent_is("fp_text") | self.parent_is("gr_text") {
            if let "effects" = ele {
                return Some(indent)
            }
        }
        if self.parent_is("pts") {
            if let "xy" = ele {
                if self.pts_xy_count > 0 && self.pts_xy_count % 4 == 0 {
                    return Some(indent)
                } else if self.pts_xy_count == 0 && (self.is("polygon") || self.is("filled_polygon") ) {
                    return Some(indent)
                }
            }
        }
        if self.parent_is("model") {
            match ele {
                "at" | "scale" | "rotate" => {
                    return Some(indent)
                },
                _ => (),
            }
        }
        if self.parent_is("pad") {
            if let "net" = ele {
                return Some(indent)
            }
        }
        None
    }
    
    fn want_indent_layout(&self, ele:&str) -> Option<Indent> {
        if !self.is("kicad_pcb") {
            return None
        }
        let mut indent = Indent::default();
        indent.before();
        if self.parent_is("kicad_pcb") {
            match ele {
                "page" | "gr_circle"
                    => {
                        indent.before_double();
                        return Some(indent)
                    },
                "module"
                    => {
                        indent.before_double();
                        indent.close_on_new_line();
                        return Some(indent)
                    },
                "net" | "gr_line" | "gr_arc" | "segment" | "via"
                    => return Some(indent),
                "layers" | "gr_text" | "dimension"
                    => {
                        indent.close_on_new_line();
                        return Some(indent)
                    },
                "setup"
                    => {
                        indent.before_double();
                        indent.close_on_new_line();
                        indent.newline_after_closing();
                        return Some(indent)
                    },
                "general" | "net_class" | "zone"
                    => {
                        indent.before_double();
                        indent.close_on_new_line();
                        return Some(indent)
                    },
                _ => (),
            }
        }
        if self.parent_is("general") {
            return Some(indent)
        }
        if self.parent_is("layers") {
            return Some(indent)
        }
        if self.parent_is("setup") {
            return Some(indent)
        }
        if self.parent_is("pcbplotparams") {
            return Some(indent)
        }
        if self.parent_is("net_class") {
            return Some(indent)
        }
        if self.parent_is("dimension") {
            match ele {
                "gr_text" | "feature1" |
                "feature2" | "crossbar" |
                "arrow1a" | "arrow1b" |
                "arrow2a" | "arrow2b" => {
                    return Some(indent)
                },
                _ => (),
            }
        }
        if self.parent_is("zone") {
            match ele {
                "connect_pads" | "min_thickness" | "fill"
                    => return Some(indent),
                "polygon" | "filled_polygon"
                    => {
                        indent.close_on_new_line();
                        return Some(indent)
                    },
                _ => (),
            }
        }
        if self.parent_is("polygon") | self.parent_is("filled_polygon") {
            return Some(indent)
        }
        None
    }
    
    fn want_indent(&self, value:&Sexp) -> Option<Indent> {
        let first = match *value {
            Sexp::List(ref l) => {
                if l.is_empty() {
                    return None
                }
                (&l[0]).clone()
            },
            Sexp::Empty => return None,
            Sexp::String(ref l) => Sexp::String(l.clone()),
        };
        if let Sexp::String(ref ele) = first {
            let i = self.want_indent_module(ele);
            if i.is_some() {
                return i
            }
            let i = self.want_indent_layout(ele);
            if i.is_some() {
                return i
            }
        }
        None
    }
}

impl Formatter for KicadFormatter {
    
    fn open<W>(&mut self, writer: &mut W, value:Option<&Sexp>) -> Result<()>
        where W: io::Write
    {
        let mut ele = String::new();
        // if first element is string
        if let Some(ref sexp) = value {
            if let Sexp::String(ref s) = **sexp {
                ele.push_str(s);
            }
        }
        let exp = Sexp::String(ele.clone());
        let want_indent = self.want_indent(&exp);
        if let Some(ref want_indent) = want_indent {
            self.indent += 1;
            if want_indent.newline_before > 0 {
                try!(self.indent(writer, want_indent.newline_before));
            }
        }
        
        // special handling for breaking after 4 elements of xy
        if let "pts" = &ele[..] {
            self.pts_xy_count = 0;
        }
        if self.parent_is("pts") {
            if let "xy" = &ele[..] {
                self.pts_xy_count += 1;
                if self.pts_xy_count == 5 {
                    self.pts_xy_count = 1;
                }
            }
        }
        
        if !ele.is_empty() {
            self.stack.push(Some((ele, want_indent)))
        } else {
            self.stack.push(None)
        }
        writer.write_all(b"(").map_err(From::from)
    }
    
    fn element<W>(&mut self, writer: &mut W, value:&Sexp) -> Result<()>
        where W: io::Write
    {
        // get rid of the space if we will be putting a newline next
        if self.want_indent(value).is_none() {
            try!(writer.write_all(b" "));
        } else if let Sexp::String(_) = *value {
            try!(writer.write_all(b" "));
        }
        Ok(())
        
    }
    
    fn close<W>(&mut self, writer: &mut W) -> Result<()>
        where W: io::Write
    {
        if let Some(Some((s, want_indent))) = self.stack.pop() {
            if let Some(indent) = want_indent {
                self.indent -= 1;
                if indent.closing_on_new_line {
                    try!(self.indent_plus(writer, 1));
                }
                // special handling of toplevel module...
                // which doesn't work, because it is not indented
                if &s == "module" && self.stack.is_empty() {
                    try!(writer.write_all(b"\n"));
                }
                try!(writer.write_all(b")"));
                for _ in 0..indent.newline_after {
                    try!(writer.write_all(b"\n"));
                }
                return Ok(())
            } else {
                if self.stack.is_empty() && (&s == "module" || &s == "kicad_pcb") {
                    try!(writer.write_all(b"\n"));
                }
            }
        }
        try!(writer.write_all(b")"));
        Ok(())
    }
}
