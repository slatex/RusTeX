pub fn pt(f:f32) -> f32 { f * 65536.0 }
pub fn inch(f:f32) -> f32 { pt(f) * 72.27 }
pub fn cm(f:f32) -> f32 { inch(f) / 2.54 }
pub fn mm(f:f32) -> f32 { cm(f) / 10.0 }

#[derive(Copy,Clone)]
pub enum SkipDim {
    Pt(i32),
    Fil(i32),
    Fill(i32),
    Filll(i32)
}
impl SkipDim {
    pub fn negate(self) -> SkipDim {
        use SkipDim::*;
        match self {
            Pt(i) => Pt(-i),
            Fil(i) => Fil(-i),
            Fill(i) => Fill(-i),
            Filll(i) => Filll(-i)
        }
    }
}
#[derive(Copy,Clone)]
pub struct Skip {
    pub base : i32,
    pub stretch : Option<SkipDim>,
    pub shrink: Option<SkipDim>
}
impl Skip {
    pub fn negate(self) -> Skip {
        Skip {
            base:-self.base,
            stretch:self.stretch.map(|x| x.negate()),
            shrink:self.shrink.map(|x| x.negate())
        }
    }
}

pub enum Numeric {
    Int(i32),
    Dim(i32),
    Float(f32),
    Skip(Skip)
}
impl Numeric {
    pub fn negate(self) -> Numeric {
        use Numeric::*;
        match self {
            Int(i) => Int(-i),
            Dim(i) => Dim(-i),
            Float(f) => Float(-f),
            Skip(sk) => Skip(sk.negate())
        }
    }
}
