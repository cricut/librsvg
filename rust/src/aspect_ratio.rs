//! Handling of preserveAspectRatio values
//!
//! This module handles preserveAspectRatio values [per the SVG specification][spec].
//! We have an [`AspectRatio`] struct which encapsulates such a value.
//!
//! [`AspectRatio`] implements `FromStr`, so it can be parsed easily:
//!
//! ```
//! assert_eq! (AspectRatio::from_str ("xMidYMid"),
//!             Ok (AspectRatio { defer: false,
//!                               align: Align::Aligned { align: AlignMode::XmidYmid,
//!                                                       fit: FitMode::Meet } }));
//! ```
//!
//! [`AspectRatio`]: struct.AspectRatio.html
//! [spec]: https://www.w3.org/TR/SVG/coords.html#PreserveAspectRatioAttribute

use ::libc;
use ::glib::translate::*;

use parsers::Parse;
use parsers::ParseError;
use error::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FitMode {
    Meet,
    Slice
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AlignMode {
    XminYmin,
    XmidYmin,
    XmaxYmin,
    XminYmid,
    XmidYmid,
    XmaxYmid,
    XminYmax,
    XmidYmax,
    XmaxYmax
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Align {
    None,
    Aligned {
        align: AlignMode,
        fit: FitMode
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct AspectRatio {
    pub defer: bool,
    pub align: Align
}

enum Align1D {
    Min,
    Mid,
    Max
}

fn align_1d (a: Align1D, dest_pos: f64, dest_size: f64, obj_size: f64) -> f64 {
    match a {
        Align1D::Min => { dest_pos },
        Align1D::Mid => { dest_pos + (dest_size - obj_size) / 2.0 },
        Align1D::Max => { dest_pos + dest_size - obj_size }
    }
}

impl AspectRatio {
    pub fn from_u32 (val: u32) -> AspectRatio {
        let val = AspectRatioFlags::from_bits (val).unwrap ();

        let defer = val.contains (DEFER);

        let mut aligned: bool = true;

        let align: AlignMode = {
            if val.contains (XMIN_YMIN)      { AlignMode::XminYmin }
            else if val.contains (XMID_YMIN) { AlignMode::XmidYmin }
            else if val.contains (XMAX_YMIN) { AlignMode::XmaxYmin }
            else if val.contains (XMIN_YMID) { AlignMode::XminYmid }
            else if val.contains (XMID_YMID) { AlignMode::XmidYmid }
            else if val.contains (XMAX_YMID) { AlignMode::XmaxYmid }
            else if val.contains (XMIN_YMAX) { AlignMode::XminYmax }
            else if val.contains (XMID_YMAX) { AlignMode::XmidYmax }
            else if val.contains (XMAX_YMAX) { AlignMode::XmaxYmax }
            else {
                aligned = false;
                AlignMode::XmidYmid
            }
        };

        let fit: FitMode = if val.contains(SLICE) { FitMode::Slice } else { FitMode::Meet };

        AspectRatio {
            defer: defer,
            align: if aligned {
                Align::Aligned {
                    align: align,
                    fit: fit
                }
            } else {
                Align::None
            }
        }
    }

    pub fn to_u32 (&self) -> u32 {
        let mut val = AspectRatioFlags::empty ();

        if self.defer { val = val | DEFER; }

        match self.align {
            Align::None => { },

            Align::Aligned { align, fit } => {
                match align {
                    AlignMode::XminYmin => { val = val | XMIN_YMIN; },
                    AlignMode::XmidYmin => { val = val | XMID_YMIN; },
                    AlignMode::XmaxYmin => { val = val | XMAX_YMIN; },
                    AlignMode::XminYmid => { val = val | XMIN_YMID; },
                    AlignMode::XmidYmid => { val = val | XMID_YMID; },
                    AlignMode::XmaxYmid => { val = val | XMAX_YMID; },
                    AlignMode::XminYmax => { val = val | XMIN_YMAX; },
                    AlignMode::XmidYmax => { val = val | XMID_YMAX; },
                    AlignMode::XmaxYmax => { val = val | XMAX_YMAX; },
                }

                match fit {
                    FitMode::Meet  => { },
                    FitMode::Slice => { val = val | SLICE; }
                }
            }
        }

        val.bits ()
    }

    pub fn compute (&self,
                    object_width: f64,
                    object_height: f64,
                    dest_x: f64,
                    dest_y: f64,
                    dest_width: f64,
                    dest_height: f64) -> (f64, f64, f64, f64) {
        match self.align {
            Align::None => { (dest_x, dest_y, dest_width, dest_height) }

            Align::Aligned { align, fit } => {
                let w_factor = dest_width / object_width;
                let h_factor = dest_height / object_height;
                let factor: f64;

                match fit {
                    FitMode::Meet  => { factor = w_factor.min (h_factor); }
                    FitMode::Slice => { factor = w_factor.max (h_factor); }
                }

                let w = object_width * factor;
                let h = object_height * factor;

                let xalign: Align1D;
                let yalign: Align1D;

                match align {
                    AlignMode::XminYmin => { xalign = Align1D::Min; yalign = Align1D::Min; },
                    AlignMode::XminYmid => { xalign = Align1D::Min; yalign = Align1D::Mid; },
                    AlignMode::XminYmax => { xalign = Align1D::Min; yalign = Align1D::Max; },
                    AlignMode::XmidYmin => { xalign = Align1D::Mid; yalign = Align1D::Min; },
                    AlignMode::XmidYmid => { xalign = Align1D::Mid; yalign = Align1D::Mid; },
                    AlignMode::XmidYmax => { xalign = Align1D::Mid; yalign = Align1D::Max; },
                    AlignMode::XmaxYmin => { xalign = Align1D::Max; yalign = Align1D::Min; },
                    AlignMode::XmaxYmid => { xalign = Align1D::Max; yalign = Align1D::Mid; },
                    AlignMode::XmaxYmax => { xalign = Align1D::Max; yalign = Align1D::Max; }
                }

                let xpos = align_1d (xalign, dest_x, dest_width, w);
                let ypos = align_1d (yalign, dest_y, dest_height, h);

                (xpos, ypos, w, h)
            }
        }
    }
}

impl Default for Align {
    fn default () -> Align {
        Align::Aligned {
            align: AlignMode::XmidYmid,
            fit: FitMode::Meet
        }
    }
}

impl Default for AspectRatio {
    fn default () -> AspectRatio {
        AspectRatio {
            defer: false,
            align: Default::default ()
        }
    }
}

bitflags! {
    struct AspectRatioFlags: u32 {
        const XMIN_YMIN = (1 << 0);
        const XMID_YMIN = (1 << 1);
        const XMAX_YMIN = (1 << 2);
        const XMIN_YMID = (1 << 3);
        const XMID_YMID = (1 << 4);
        const XMAX_YMID = (1 << 5);
        const XMIN_YMAX = (1 << 6);
        const XMID_YMAX = (1 << 7);
        const XMAX_YMAX = (1 << 8);
        const SLICE = (1 << 30);
        const DEFER = (1 << 31);
    }
}


fn parse_align_mode (s: &str) -> Option<Align> {
    match s {
        "none"     => { Some (Align::None) },
        "xMinYMin" => { Some (Align::Aligned { align: AlignMode::XminYmin, fit: FitMode::Meet } ) },
        "xMidYMin" => { Some (Align::Aligned { align: AlignMode::XmidYmin, fit: FitMode::Meet } ) },
        "xMaxYMin" => { Some (Align::Aligned { align: AlignMode::XmaxYmin, fit: FitMode::Meet } ) },
        "xMinYMid" => { Some (Align::Aligned { align: AlignMode::XminYmid, fit: FitMode::Meet } ) },
        "xMidYMid" => { Some (Align::Aligned { align: AlignMode::XmidYmid, fit: FitMode::Meet } ) },
        "xMaxYMid" => { Some (Align::Aligned { align: AlignMode::XmaxYmid, fit: FitMode::Meet } ) },
        "xMinYMax" => { Some (Align::Aligned { align: AlignMode::XminYmax, fit: FitMode::Meet } ) },
        "xMidYMax" => { Some (Align::Aligned { align: AlignMode::XmidYmax, fit: FitMode::Meet } ) },
        "xMaxYMax" => { Some (Align::Aligned { align: AlignMode::XmaxYmax, fit: FitMode::Meet } ) },
        _          => { None }
    }
}

fn parse_fit_mode (s: &str) -> Option<FitMode> {
    match s {
        "meet"  => { Some (FitMode::Meet) },
        "slice" => { Some (FitMode::Slice) },
        _       => { None }
    }
}

enum ParseState {
    Defer,
    Align,
    Fit,
    Finished
}

fn make_err () -> AttributeError {
    AttributeError::Parse (ParseError::new ("expected \"[defer] <align> [meet | slice]\""))
}

impl Parse for AspectRatio {
    type Data = ();
    type Err = AttributeError;

    fn parse(s: &str, _: ()) -> Result<AspectRatio, AttributeError> {
        let mut defer = false;
        let mut align: Align = Default::default ();
        let mut fit_mode = FitMode::Meet;

        let mut state = ParseState::Defer;
        let mut iter = s.split_whitespace ();

        while let Some (v) = iter.next () {
            match state {
                ParseState::Defer => {
                    if v == "defer" {
                        defer = true;
                        state = ParseState::Align;
                    } else if let Some (parsed_align) = parse_align_mode (v) {
                        align = parsed_align;
                        state = ParseState::Fit;
                    } else {
                        return Err(make_err ());
                    }
                },

                ParseState::Align => {
                    if let Some (parsed_align) = parse_align_mode (v) {
                        align = parsed_align;
                        state = ParseState::Fit;
                    } else {
                        return Err(make_err ());
                    }
                },

                ParseState::Fit => {
                    if let Some (parsed_fit) = parse_fit_mode (v) {
                        fit_mode = parsed_fit;
                        state = ParseState::Finished;
                    } else {
                        return Err(make_err ());
                    }
                },

                _ => {
                    return Err(make_err ());
                }
            }
        }

        // The string must match "[defer] <align> [meet | slice]".
        // Since the meet|slice is optional, we can end up in either
        // of the following states:
        match state {
            ParseState::Fit | ParseState::Finished => {},
            _ => { return Err(make_err ()); }
        }

        Ok (AspectRatio {
            defer: defer,
            align: match align {
                Align::None => { Align::None },
                Align::Aligned { align, .. } => {
                    Align::Aligned {
                        align: align,
                        fit: fit_mode
                    }
                }
            }
        })
    }
}

#[no_mangle]
pub extern fn rsvg_aspect_ratio_parse (c_str: *const libc::c_char) -> u32 {
    let my_str = unsafe { &String::from_glib_none (c_str) };
    let parsed = AspectRatio::parse (my_str, ());

    match parsed {
        Ok (aspect_ratio) => { aspect_ratio.to_u32 () },
        Err (_) => {
            // We can't propagate the error here, so just return a default value
            let a: AspectRatio = Default::default ();
            a.to_u32 ()
        }
    }
}

#[no_mangle]
pub extern fn rsvg_aspect_ratio_compute (aspect: u32,
                                         object_width: f64,
                                         object_height: f64,
                                         dest_x: *mut f64,
                                         dest_y: *mut f64,
                                         dest_width: *mut f64,
                                         dest_height: *mut f64) {
    unsafe {
        let (x, y, w, h) = AspectRatio::from_u32 (aspect).compute (object_width, object_height, *dest_x, *dest_y, *dest_width, *dest_height);
        *dest_x = x;
        *dest_y = y;
        *dest_width = w;
        *dest_height = h;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing_invalid_strings_yields_error () {
        assert! (AspectRatio::parse ("", ()).is_err ());

        assert! (AspectRatio::parse ("defer", ()).is_err ());

        assert! (AspectRatio::parse ("defer foo", ()).is_err ());

        assert! (AspectRatio::parse ("defer xmidymid", ()).is_err ());

        assert! (AspectRatio::parse ("defer xMidYMid foo", ()).is_err ());

        assert! (AspectRatio::parse ("xmidymid", ()).is_err ());

        assert! (AspectRatio::parse ("xMidYMid foo", ()).is_err ());

        assert! (AspectRatio::parse ("defer xMidYMid meet foo", ()).is_err ());
    }

    #[test]
    fn parses_valid_strings () {
        assert_eq! (AspectRatio::parse ("defer none", ()),
                    Ok (AspectRatio { defer: true,
                                      align: Align::None }));

        assert_eq! (AspectRatio::parse ("xMidYMid", ()),
                    Ok (AspectRatio { defer: false,
                                      align: Align::Aligned { align: AlignMode::XmidYmid,
                                                              fit: FitMode::Meet } }));
        
        assert_eq! (AspectRatio::parse ("defer xMidYMid", ()),
                    Ok (AspectRatio { defer: true,
                                      align: Align::Aligned { align: AlignMode::XmidYmid,
                                                              fit: FitMode::Meet } }));
        
        assert_eq! (AspectRatio::parse ("defer xMinYMax", ()),
                    Ok (AspectRatio { defer: true,
                                      align: Align::Aligned { align: AlignMode::XminYmax,
                                                              fit: FitMode::Meet } }));
        
        assert_eq! (AspectRatio::parse ("defer xMaxYMid meet", ()),
                    Ok (AspectRatio { defer: true,
                                      align: Align::Aligned { align: AlignMode::XmaxYmid,
                                                              fit: FitMode::Meet } }));
        
        assert_eq! (AspectRatio::parse ("defer xMinYMax slice", ()),
                    Ok (AspectRatio { defer: true,
                                      align: Align::Aligned { align: AlignMode::XminYmax,
                                                              fit: FitMode::Slice } }));
    }

    fn test_roundtrip (s: &str) {
        let a = AspectRatio::parse (s, ()).unwrap ();

        assert_eq! (AspectRatio::from_u32 (a.to_u32 ()), a);
    }

    #[test]
    fn conversion_to_u32_roundtrips () {
        test_roundtrip ("defer xMidYMid");
        test_roundtrip ("defer xMinYMax slice");
        test_roundtrip ("xMaxYMax meet");
        test_roundtrip ("xMinYMid slice");
    }

    #[test]
    fn aligns () {
        assert_eq! (AspectRatio::parse ("xMinYMin meet", ()).unwrap().compute (1.0, 10.0, 0.0, 0.0, 10.0, 1.0), (0.0, 0.0, 0.1, 1.0));
        assert_eq! (AspectRatio::parse ("xMinYMin slice", ()).unwrap().compute (1.0, 10.0, 0.0, 0.0, 10.0, 1.0), (0.0, 0.0, 10.0, 100.0));

        assert_eq! (AspectRatio::parse ("xMinYMid meet", ()).unwrap().compute (1.0, 10.0, 0.0, 0.0, 10.0, 1.0), (0.0, 0.0, 0.1, 1.0));
        assert_eq! (AspectRatio::parse ("xMinYMid slice", ()).unwrap().compute (1.0, 10.0, 0.0, 0.0, 10.0, 1.0), (0.0, -49.5, 10.0, 100.0));

        assert_eq! (AspectRatio::parse ("xMinYMax meet", ()).unwrap().compute (1.0, 10.0, 0.0, 0.0, 10.0, 1.0), (0.0, 0.0, 0.1, 1.0));
        assert_eq! (AspectRatio::parse ("xMinYMax slice", ()).unwrap().compute (1.0, 10.0, 0.0, 0.0, 10.0, 1.0), (0.0, -99.0, 10.0, 100.0));

        assert_eq! (AspectRatio::parse ("xMidYMin meet", ()).unwrap().compute (1.0, 10.0, 0.0, 0.0, 10.0, 1.0), (4.95, 0.0, 0.1, 1.0));
        assert_eq! (AspectRatio::parse ("xMidYMin slice", ()).unwrap().compute (1.0, 10.0, 0.0, 0.0, 10.0, 1.0), (0.0, 0.0, 10.0, 100.0));

        assert_eq! (AspectRatio::parse ("xMidYMid meet", ()).unwrap().compute (1.0, 10.0, 0.0, 0.0, 10.0, 1.0), (4.95, 0.0, 0.1, 1.0));
        assert_eq! (AspectRatio::parse ("xMidYMid slice", ()).unwrap().compute (1.0, 10.0, 0.0, 0.0, 10.0, 1.0), (0.0, -49.5, 10.0, 100.0));

        assert_eq! (AspectRatio::parse ("xMidYMax meet", ()).unwrap().compute (1.0, 10.0, 0.0, 0.0, 10.0, 1.0), (4.95, 0.0, 0.1, 1.0));
        assert_eq! (AspectRatio::parse ("xMidYMax slice", ()).unwrap().compute (1.0, 10.0, 0.0, 0.0, 10.0, 1.0), (0.0, -99.0, 10.0, 100.0));

        assert_eq! (AspectRatio::parse ("xMaxYMin meet", ()).unwrap().compute (1.0, 10.0, 0.0, 0.0, 10.0, 1.0), (9.9, 0.0, 0.1, 1.0));
        assert_eq! (AspectRatio::parse ("xMaxYMin slice", ()).unwrap().compute (1.0, 10.0, 0.0, 0.0, 10.0, 1.0), (0.0, 0.0, 10.0, 100.0));

        assert_eq! (AspectRatio::parse ("xMaxYMid meet", ()).unwrap().compute (1.0, 10.0, 0.0, 0.0, 10.0, 1.0), (9.9, 0.0, 0.1, 1.0));
        assert_eq! (AspectRatio::parse ("xMaxYMid slice", ()).unwrap().compute (1.0, 10.0, 0.0, 0.0, 10.0, 1.0), (0.0, -49.5, 10.0, 100.0));

        assert_eq! (AspectRatio::parse ("xMaxYMax meet", ()).unwrap().compute (1.0, 10.0, 0.0, 0.0, 10.0, 1.0), (9.9, 0.0, 0.1, 1.0));
        assert_eq! (AspectRatio::parse ("xMaxYMax slice", ()).unwrap().compute (1.0, 10.0, 0.0, 0.0, 10.0, 1.0), (0.0, -99.0, 10.0, 100.0));
    }
}
