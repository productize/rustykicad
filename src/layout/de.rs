// (c) 2016 Productize SPRL <joost@productize.be>

// extension: .kicad_pcb
// format: new-style

// from parent
use footprint;
use wrap;
use Sexp;
use GrElement;
use symbolic_expressions::iteratom::*;

use layout::data::*;

// TODO: switch more to IterAtom like in footprint/de.rs

struct Version(i64);

impl FromSexp for Version {
    fn from_sexp(s: &Sexp) -> SResult<Version> {
        let mut i = IterAtom::new(s, "version")?;
        Ok(Version(i.i("value")?))
    }
}

struct Page(String);

impl FromSexp for Page {
    fn from_sexp(s: &Sexp) -> SResult<Page> {
        let mut i = IterAtom::new(s, "page")?;
        Ok(Page(i.s("value")?))
    }
}

struct Polygon(footprint::Pts);

impl FromSexp for Polygon {
    fn from_sexp(s: &Sexp) -> SResult<Polygon> {
        let mut i = IterAtom::new(s, "polygon")?;
        Ok(Polygon(i.t("pts")?))
    }
}

struct FilledPolygon(footprint::Pts);

impl FromSexp for FilledPolygon {
    fn from_sexp(s: &Sexp) -> SResult<FilledPolygon> {
        let mut i = IterAtom::new(s, "filled_polygon")?;
        Ok(FilledPolygon(i.t("pts")?))
    }
}

struct FillSegments(footprint::Pts);

impl FromSexp for FillSegments {
    fn from_sexp(s: &Sexp) -> SResult<FillSegments> {
        let mut i = IterAtom::new(s, "fill_segments")?;
        Ok(FillSegments(i.t("pts")?))
    }
}

impl FromSexp for Net {
    fn from_sexp(s: &Sexp) -> SResult<Net> {
        let mut i = IterAtom::new(s, "net")?;
        let num = i.i("num")?;
        let name = i.s("name")?;
        Ok(Net {
            name: name,
            num: num,
        })
    }
}

impl FromSexp for Host {
    fn from_sexp(s: &Sexp) -> SResult<Host> {
        let mut i = IterAtom::new(s, "host")?;
        let tool = i.s("tool")?;
        let build = i.s("build")?;
        Ok(Host {
            tool: tool,
            build: build,
        })
    }
}

impl FromSexp for General {
    fn from_sexp(s: &Sexp) -> SResult<General> {
        let mut i = IterAtom::new(s, "general")?;
        let links = i.i_in_list("links")?;
        let no_connects = i.i_in_list("no_connects")?;
        let area: Area = i.t("area")?;
        let thickness = i.f_in_list("thickness")?;
        let drawings = i.i_in_list("drawings")?;
        let tracks = i.i_in_list("tracks")?;
        let zones = i.i_in_list("zones")?;
        let modules = i.i_in_list("modules")?;
        let nets = i.i_in_list("nets")?;
        Ok(General {
            links: links,
            no_connects: no_connects,
            area: area,
            thickness: thickness,
            drawings: drawings,
            tracks: tracks,
            zones: zones,
            modules: modules,
            nets: nets,
        })
    }
}

impl FromSexp for Area {
    fn from_sexp(s: &Sexp) -> SResult<Area> {
        let l = s.slice_atom_num("area", 4)?;
        let x1 = l[0].f()?;
        let y1 = l[1].f()?;
        let x2 = l[2].f()?;
        let y2 = l[3].f()?;
        Ok(Area {
            x1: x1,
            y1: y1,
            x2: x2,
            y2: y2,
        })
    }
}

struct LayerVec(Vec<Layer>);

impl FromSexp for LayerVec {
    fn from_sexp(s: &Sexp) -> SResult<LayerVec> {
        let mut v = vec![];
        let l = s.slice_atom("layers")?;
        for x in l {
            let layer = from_sexp(&x)?;
            v.push(layer)
        }
        Ok(LayerVec(v))
    }
}

impl FromSexp for Layer {
    fn from_sexp(s: &Sexp) -> SResult<Layer> {
        let l = s.list()?;
        // println!("making layer from {}", s);
        if l.len() != 3 && l.len() != 4 {
            return Err(format!("expecting 3 or 4 elements in layer: {}", s).into());
        }
        let num = l[0].i()?;
        let layer = footprint::Layer::from_string(l[1].string()?)?;
        let layer_type = from_sexp(&l[2])?;
        let hide = if l.len() == 3 {
            false
        } else {
            let h = l[3].string()?;
            match &h[..] {
                "hide" => true,
                _ => false,
            }
        };
        Ok(Layer {
            num: num,
            layer: layer,
            layer_type: layer_type,
            hide: hide,
        })
    }
}

impl FromSexp for LayerType {
    fn from_sexp(s: &Sexp) -> SResult<LayerType> {
        let x = s.string()?;
        match &x[..] {
            "signal" => Ok(LayerType::Signal),
            "user" => Ok(LayerType::User),
            _ => Err(format!("unknown layertype {} in {}", x, s).into()),
        }
    }
}

impl FromSexp for SetupElement {
    fn from_sexp(s: &Sexp) -> SResult<SetupElement> {
        let l = s.list()?;
        if l.len() != 2 && l.len() != 3 {
            return Err(format!("expecting 2 or 3 elements in setup element: {}", s).into());
        }
        let name = l[0].string()?.clone();
        let value1 = l[1].string()?.clone();
        let value2 = match l.len() {
            3 => Some(l[2].string()?.clone()),
            _ => None,
        };
        Ok(SetupElement {
            name: name,
            value1: value1,
            value2: value2,
        })
    }
}

impl FromSexp for NetClass {
    fn from_sexp(s: &Sexp) -> SResult<NetClass> {
        fn parse(e: &Sexp, name: &str) -> SResult<f64> {
            let l = e.slice_atom(name)?;
            l[0].f().map_err(From::from)
        }
        let l = s.slice_atom("net_class")?;
        let name = l[0].string()?.clone();
        let desc = l[1].string()?.clone();
        let mut clearance = 0.1524;
        let mut trace_width = 0.2032;
        let mut via_dia = 0.675;
        let mut via_drill = 0.25;
        let mut uvia_dia = 0.508;
        let mut uvia_drill = 0.127;
        let mut diff_pair_gap = None;
        let mut diff_pair_width = None;
        let mut nets = vec![];
        for x in &l[2..] {
            let list_name = x.list_name()?;
            let xn = &list_name[..];
            match xn {
                "add_net" => {
                    let l1 = x.slice_atom("add_net")?;
                    nets.push(l1[0].string()?.clone())
                }
                "clearance" => clearance = parse(x, xn)?,
                "trace_width" => trace_width = parse(x, xn)?,
                "via_dia" => via_dia = parse(x, xn)?,
                "via_drill" => via_drill = parse(x, xn)?,
                "uvia_dia" => uvia_dia = parse(x, xn)?,
                "uvia_drill" => uvia_drill = parse(x, xn)?,
                "diff_pair_gap" => diff_pair_gap = Some(parse(x, xn)?),
                "diff_pair_width" => diff_pair_width = Some(parse(x, xn)?),
                _ => return Err(format!("unknown net_class field {}", list_name).into()),
            }
        }
        let net_class = NetClass {
            name: name,
            desc: desc,
            clearance: clearance,
            via_dia: via_dia,
            via_drill: via_drill,
            uvia_dia: uvia_dia,
            uvia_drill: uvia_drill,
            diff_pair_gap: diff_pair_gap,
            diff_pair_width: diff_pair_width,
            nets: nets,
            trace_width: trace_width,
        };
        Ok(net_class)
    }
}

impl FromSexp for Setup {
    fn from_sexp(s: &Sexp) -> SResult<Setup> {
        let mut elements = vec![];
        let mut pcbplotparams = vec![];
        for v in s.slice_atom("setup")? {
            let n = v.list_name().unwrap().clone();
            match &n[..] {
                "pcbplotparams" => {
                    for y in v.slice_atom("pcbplotparams")? {
                        let p_e = from_sexp(y)?;
                        pcbplotparams.push(p_e)
                    }
                }
                _ => {
                    let setup_element = from_sexp(&v)?;
                    elements.push(setup_element)
                }
            }
        }
        let s = Setup {
            elements: elements,
            pcbplotparams: pcbplotparams,
        };
        Ok(s)
    }
}

// for some reason this needs to be in a subfunction or it doesn't work
fn parse_other(e: &Sexp) -> Element {
    let e2 = e.clone();
    debug!("Element::Other: {}", e2);
    Element::Other(e2)
}

impl FromSexp for GrText {
    fn from_sexp(s: &Sexp) -> SResult<GrText> {
        let l = s.slice_atom("gr_text")?;
        let value = l[0].string()?.clone();
        let mut layer = footprint::Layer::default();
        let mut tstamp = None;
        let mut at = footprint::At::default();
        let mut effects = footprint::Effects::default();
        for x in &l[1..] {
            let elem = from_sexp(x)?;
            match elem {
                GrElement::At(x) => at = x,
                GrElement::Layer(x) => layer = x,
                GrElement::TStamp(x) => tstamp = Some(x),
                GrElement::Effects(x) => effects = x,
                _ => (), // TODO
            }
        }
        Ok(GrText {
            value: value,
            at: at,
            layer: layer,
            effects: effects,
            tstamp: tstamp,
        })
    }
}

impl FromSexp for GrElement {
    fn from_sexp(s: &Sexp) -> SResult<GrElement> {
        match &(s.list_name()?)[..] {
            "start" => wrap(s, from_sexp, GrElement::Start),
            "end" => wrap(s, from_sexp, GrElement::End),
            "center" => wrap(s, from_sexp, GrElement::Center),
            "angle" => {
                let l2 = s.slice_atom("angle")?;
                Ok(GrElement::Angle(l2[0].f()?))
            }
            "layer" => wrap(s, from_sexp, GrElement::Layer),
            "width" => {
                let l2 = s.slice_atom("width")?;
                Ok(GrElement::Width(l2[0].f()?))
            }
            "size" => {
                let l2 = s.slice_atom("size")?;
                Ok(GrElement::Size(l2[0].f()?))
            }
            "drill" => {
                let l2 = s.slice_atom("drill")?;
                Ok(GrElement::Drill(l2[0].f()?))
            }
            "tstamp" => {
                let l2 = s.slice_atom("tstamp")?;
                let sx = l2[0].string()?.clone();
                Ok(GrElement::TStamp(sx))
            }
            "status" => {
                let l2 = s.slice_atom("status")?;
                let sx = l2[0].string()?.clone();
                Ok(GrElement::Status(sx))
            }
            "net" => {
                let l2 = s.slice_atom("net")?;
                Ok(GrElement::Net(l2[0].i()?))
            }
            "at" => wrap(s, from_sexp, GrElement::At),
            "layers" => wrap(s, from_sexp, GrElement::Layers),
            "effects" => wrap(s, from_sexp, GrElement::Effects),
            x => Err(format!("unknown element {} in {}", x, s).into()),
        }
    }
}


impl FromSexp for GrLine {
    fn from_sexp(s: &Sexp) -> SResult<GrLine> {
        // println!("GrLine: {}", s);
        let l = s.slice_atom("gr_line")?;
        let mut start = footprint::Xy::new_empty(footprint::XyType::Start);
        let mut end = footprint::Xy::new_empty(footprint::XyType::End);
        let mut angle = 0.0_f64;
        let mut layer = footprint::Layer::default();
        let mut width = 0.0_f64;
        let mut tstamp = None;
        for x in l {
            let elem = from_sexp(x)?;
            match elem {
                GrElement::Start(x) => start = x,
                GrElement::End(x) => end = x,
                GrElement::Angle(x) => angle = x,
                GrElement::Layer(x) => layer = x,
                GrElement::TStamp(x) => tstamp = Some(x),
                GrElement::Width(x) => width = x,
                _ => (), // TODO
            }
        }
        Ok(GrLine {
            start: start,
            end: end,
            angle: angle,
            layer: layer,
            width: width,
            tstamp: tstamp,
        })
    }
}

impl FromSexp for GrArc {
    fn from_sexp(s: &Sexp) -> SResult<GrArc> {
        let l = s.slice_atom("gr_arc")?;
        let mut start = footprint::Xy::new_empty(footprint::XyType::Start);
        let mut end = footprint::Xy::new_empty(footprint::XyType::End);
        let mut angle = 0.0_f64;
        let mut layer = footprint::Layer::default();
        let mut width = 0.0_f64;
        let mut tstamp = None;
        for x in l {
            let elem = from_sexp(x)?;
            match elem {
                GrElement::Start(x) => start = x,
                GrElement::End(x) => end = x,
                GrElement::Angle(x) => angle = x,
                GrElement::Layer(x) => layer = x,
                GrElement::TStamp(x) => tstamp = Some(x),
                GrElement::Width(x) => width = x,
                _ => (), // TODO
            }
        }
        Ok(GrArc {
            start: start,
            end: end,
            angle: angle,
            layer: layer,
            width: width,
            tstamp: tstamp,
        })
    }
}

impl FromSexp for GrCircle {
    fn from_sexp(s: &Sexp) -> SResult<GrCircle> {
        let l = s.slice_atom("gr_circle")?;
        let mut center = footprint::Xy::new_empty(footprint::XyType::Center);
        let mut end = footprint::Xy::new_empty(footprint::XyType::End);
        let mut layer = footprint::Layer::default();
        let mut width = 0.0_f64;
        let mut tstamp = None;
        for x in l {
            let elem = from_sexp(x)?;
            match elem {
                GrElement::Center(x) => center = x,
                GrElement::End(x) => end = x,
                GrElement::Layer(x) => layer = x,
                GrElement::TStamp(x) => tstamp = Some(x),
                GrElement::Width(x) => width = x,
                _ => (), // TODO
            }
        }
        Ok(GrCircle {
            center: center,
            end: end,
            layer: layer,
            width: width,
            tstamp: tstamp,
        })
    }
}


impl FromSexp for Dimension {
    fn from_sexp(s: &Sexp) -> SResult<Dimension> {
        let l = s.slice_atom_num("dimension", 11)?;
        let name = l[0].string()?.clone();
        let width = {
            let l2 = l[1].slice_atom("width")?;
            l2[0].f()?
        };
        let layer = from_sexp(&l[2])?;
        let (i, tstamp) = match l[3].named_value_string("tstamp") {
            Ok(s) => (4, Some(s.clone())),
            _ => (3, None),
        };
        let text = from_sexp(&l[i])?;
        let feature1 = from_sexp(l[i + 1].named_value("feature1")?)?;
        let feature2 = from_sexp(l[i + 2].named_value("feature2")?)?;
        let crossbar = from_sexp(l[i + 3].named_value("crossbar")?)?;
        let arrow1a = from_sexp(l[i + 4].named_value("arrow1a")?)?;
        let arrow1b = from_sexp(l[i + 5].named_value("arrow1b")?)?;
        let arrow2a = from_sexp(l[i + 6].named_value("arrow2a")?)?;
        let arrow2b = from_sexp(l[i + 7].named_value("arrow2b")?)?;
        Ok(Dimension {
            name: name,
            width: width,
            layer: layer,
            tstamp: tstamp,
            text: text,
            feature1: feature1,
            feature2: feature2,
            crossbar: crossbar,
            arrow1a: arrow1a,
            arrow1b: arrow1b,
            arrow2a: arrow2a,
            arrow2b: arrow2b,
        })
    }
}

impl FromSexp for Zone {
    fn from_sexp(s: &Sexp) -> SResult<Zone> {
        let mut i = IterAtom::new(s, "zone")?;
        let net = i.i_in_list("net")?;
        let net_name = i.s_in_list("net_name")?;
        let layer = i.t("layer")?;
        let tstamp = i.s_in_list("tstamp")?;
        let hatch = i.t("hatch")?;
        let priority = i.maybe_i_in_list("priority");
        let priority = match priority {
            Some(p) => p as u64,
            None => 0_u64,
        };
        let connect_pads = i.t("connect_pads")?;
        let min_thickness = i.f_in_list("min_thickness")?;
        let keepout = i.maybe_t();
        let fill = i.t("fill")?;
        let mut polygons = vec![];
        let mut filled_polygons = vec![];
        let mut fill_segments = None;
        let mut others = vec![];
        for x in i.iter {
            if let Ok(p) = Polygon::from_sexp(x) {
                polygons.push(p.0)
            } else if let Ok(p) = FilledPolygon::from_sexp(x) {
                filled_polygons.push(p.0)
            } else if let Ok(p) = FillSegments::from_sexp(x) {
                fill_segments = Some(p.0);
            } else {
                others.push(x.clone());
                debug!("'zone': not parsing {}", x);
            }
        }
        Ok(Zone {
            net: net,
            net_name: net_name,
            layer: layer,
            tstamp: tstamp,
            hatch: hatch,
            priority: priority,
            connect_pads: connect_pads,
            min_thickness: min_thickness,
            keepout: keepout,
            fill: fill,
            polygons: polygons,
            filled_polygons: filled_polygons,
            fill_segments: fill_segments,
            other: others,
        })
    }
}

impl FromSexp for Hatch {
    fn from_sexp(s: &Sexp) -> SResult<Hatch> {
        let l = s.slice_atom_num("hatch", 2)?;
        let style = l[0].string()?.clone();
        let pitch = l[1].f()?;
        Ok(Hatch {
            style: style,
            pitch: pitch,
        })
    }
}

impl FromSexp for ConnectPads {
    fn from_sexp(s: &Sexp) -> SResult<ConnectPads> {
        let l = s.slice_atom("connect_pads")?;
        let (connection, clearance) = if l.len() == 1 {
            (None, l[0].named_value_f("clearance")?)
        } else if l.len() == 2 {
            (Some(l[0].string()?.clone()), l[1].named_value_f("clearance")?)
        } else {
            return Err("unknown extra elements in connect_pads".into());
        };
        Ok(ConnectPads {
            connection: connection,
            clearance: clearance,
        })
    }
}
impl FromSexp for Keepout {
    fn from_sexp(s: &Sexp) -> SResult<Keepout> {
        let l = s.slice_atom_num("keepout", 3)?;
        let tracks = !l[0].named_value_string("tracks")?.starts_with("not");
        let vias = !l[1].named_value_string("vias")?.starts_with("not");
        let copperpour = !l[2].named_value_string("copperpour")?.starts_with("not");
        Ok(Keepout {
            tracks: tracks,
            vias: vias,
            copperpour: copperpour,
        })
    }
}

//  (fill yes (arc_segments 16) (thermal_gap 0.508) (thermal_bridge_width 0.508))
impl FromSexp for Fill {
    fn from_sexp(s: &Sexp) -> SResult<Fill> {
        let mut i = IterAtom::new(s, "fill")?;
        let filled = i.maybe_s().is_some();
        let mode = i.maybe_s_in_list("mode").is_some();
        let arc_segments = i.i_in_list("arc_segments")?;
        let thermal_gap = i.f_in_list("thermal_gap")?;
        let thermal_bridge_width = i.f_in_list("thermal_bridge_width")?;
        let smoothing = i.maybe_s_in_list("smoothing");
        let radius = i.maybe_f_in_list("radius").unwrap_or(0.0);
        Ok(Fill {
            filled: filled,
            segment: mode,
            arc_segments: arc_segments,
            thermal_gap: thermal_gap,
            thermal_bridge_width: thermal_bridge_width,
            smoothing: smoothing,
            corner_radius: radius,
        })
    }
}

impl FromSexp for Segment {
    fn from_sexp(s: &Sexp) -> SResult<Segment> {
        let i = IterAtom::new(s, "segment")?;
        let mut start = footprint::Xy::new_empty(footprint::XyType::Start);
        let mut end = footprint::Xy::new_empty(footprint::XyType::End);
        let mut layer = footprint::Layer::default();
        let mut width = 0.0_f64;
        let mut tstamp = None;
        let mut net = 0;
        let mut status = None;
        // TODO: perhaps we can get rid of GrElement now we have IterAtom...
        for x in i.iter {
            let elem = from_sexp(x)?;
            match elem {
                GrElement::Start(x) => start = x,
                GrElement::End(x) => end = x,
                GrElement::Layer(x) => layer = x,
                GrElement::TStamp(x) => tstamp = Some(x),
                GrElement::Width(x) => width = x,
                GrElement::Net(x) => net = x,
                GrElement::Status(x) => status = Some(x),
                _ => (), // TODO
            }
        }
        Ok(Segment {
            start: start,
            end: end,
            width: width,
            layer: layer,
            net: net,
            tstamp: tstamp,
            status: status,
        })
    }
}
impl FromSexp for Via {
    fn from_sexp(s: &Sexp) -> SResult<Via> {
        let i = IterAtom::new(s, "via")?;
        let mut at = footprint::At::default();
        let mut size = 0.0_f64;
        let mut drill = 0.0_f64;
        let mut layers = footprint::Layers::default();
        let mut net = 0;
        for x in i.iter {
            let elem = from_sexp(x)?;
            match elem {
                GrElement::At(x) => at = x,
                GrElement::Size(x) => size = x,
                GrElement::Net(x) => net = x,
                GrElement::Drill(x) => drill = x,
                GrElement::Layers(x) => layers = x,
                _ => (), // TODO
            }
        }
        Ok(Via {
            at: at,
            size: size,
            drill: drill,
            layers: layers,
            net: net,
        })
    }
}

impl FromSexp for Layout {
    fn from_sexp(s: &Sexp) -> SResult<Layout> {
        let i = IterAtom::new(s, "kicad_pcb")?;
        let mut layout = Layout::default();
        for e in i.iter {
            match &(e.list_name()?)[..] {
                "version" => layout.version = Version::from_sexp(e)?.0,
                "host" => layout.host = from_sexp(e)?,
                "general" => layout.general = from_sexp(&e)?,
                "page" => layout.page = Page::from_sexp(&e)?.0,
                "layers" => layout.layers = LayerVec::from_sexp(e)?.0,
                "module" => {
                    let module = wrap(e, from_sexp, Element::Module)?;
                    layout.elements.push(module)
                }
                "net" => {
                    let net = wrap(e, from_sexp, Element::Net)?;
                    layout.elements.push(net)
                }
                "net_class" => {
                    let nc = wrap(e, from_sexp, Element::NetClass)?;
                    layout.elements.push(nc)
                }
                "gr_text" => {
                    let g = wrap(e, from_sexp, Element::GrText)?;
                    layout.elements.push(g)
                }
                "gr_line" => {
                    let g = wrap(e, from_sexp, Element::GrLine)?;
                    layout.elements.push(g)
                }
                "gr_arc" => {
                    let g = wrap(e, from_sexp, Element::GrArc)?;
                    layout.elements.push(g)
                }
                "gr_circle" => {
                    let g = wrap(e, from_sexp, Element::GrCircle)?;
                    layout.elements.push(g)
                }
                "dimension" => {
                    let g = wrap(e, from_sexp, Element::Dimension)?;
                    layout.elements.push(g)
                }
                "zone" => {
                    let g = wrap(e, from_sexp, Element::Zone)?;
                    layout.elements.push(g)
                }
                "segment" => {
                    let g = wrap(e, from_sexp, Element::Segment)?;
                    layout.elements.push(g)
                }
                "via" => {
                    let g = wrap(e, from_sexp, Element::Via)?;
                    layout.elements.push(g)
                }
                "setup" => layout.setup = from_sexp(&e)?,
                _ => {
                    // println!("unimplemented: {}", e);
                    layout.elements.push(parse_other(e))
                }
            }
        }
        Ok(layout)
    }
}
