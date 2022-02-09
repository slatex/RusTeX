use std::fmt::{Display, Formatter};
use std::ops::{Add, Div};

pub fn pt(f:f32) -> f32 { f * 65536.0 }
pub fn inch(f:f32) -> f32 { pt(f) * 72.27 }
pub fn pc(f:f32) -> f32 { pt(f) * 12.0 }
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
    pub fn tomu(self) -> MuSkipDim {
        match self {
            SkipDim::Pt(i) => MuSkipDim::Mu(i),
            SkipDim::Fil(i) => MuSkipDim::Fil(i),
            SkipDim::Fill(i) => MuSkipDim::Fill(i),
            SkipDim::Filll(i) => MuSkipDim::Filll(i)
        }
    }
    pub fn negate(self) -> SkipDim {
        use SkipDim::*;
        match self {
            Pt(i) => Pt(-i),
            Fil(i) => Fil(-i),
            Fill(i) => Fill(-i),
            Filll(i) => Filll(-i)
        }
    }
    pub fn to_string(&self) -> String {
        use SkipDim::*;
        match self {
            Pt(i) =>
                ((*i as f32) / 65536.0).to_string() + "pt",
            Fil(i) =>
                ((*i as f32) / 65536.0).to_string() + "fil",
            Fill(i) =>
                ((*i as f32) / 65536.0).to_string() + "fill",
            Filll(i) =>
                ((*i as f32) / 65536.0).to_string() + "filll",
        }
    }
}
#[derive(Copy,Clone)]
pub struct Skip {
    pub base : i32,
    pub stretch : Option<SkipDim>,
    pub shrink: Option<SkipDim>
}
impl Display for Skip {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",self.to_string())
    }
}
impl Div<i32> for Skip {
    type Output = Skip;
    fn div(self, rhs: i32) -> Self::Output {
        Skip {
            base:round_f((self.base as f32) / (rhs as f32)),
            stretch:self.stretch,shrink:self.shrink
        }
    }
}
impl Skip {
    pub fn negate(self) -> Skip {
        Skip {
            base:-self.base,
            stretch:self.stretch.map(|x| x.negate()),
            shrink:self.shrink.map(|x| x.negate())
        }
    }
    pub fn to_string(&self) -> String {
        dimtostr(self.base) + &match self.stretch {
            None => "".to_string(),
            Some(s) => " plus ".to_string() + &s.to_string()
        } + &match self.shrink {
            None => "".to_string(),
            Some(s) => " minus ".to_string() + &s.to_string()
        }
    }
}

impl std::ops::Add for Skip {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Skip {
            base:self.base + rhs.base,
            stretch:self.stretch,
            shrink:self.shrink
        }
    }
}


#[derive(Copy,Clone)]
pub enum MuSkipDim {
    Mu(i32),
    Fil(i32),
    Fill(i32),
    Filll(i32)
}
impl MuSkipDim {
    pub fn negate(self) -> MuSkipDim {
        use MuSkipDim::*;
        match self {
            Mu(i) => Mu(-i),
            Fil(i) => Fil(-i),
            Fill(i) => Fill(-i),
            Filll(i) => Filll(-i)
        }
    }
    pub fn to_string(&self) -> String {
        use MuSkipDim::*;
        match self {
            Mu(i) =>
                ((*i as f32) / 65536.0).to_string() + "mu",
            Fil(i) =>
                ((*i as f32) / 65536.0).to_string() + "fil",
            Fill(i) =>
                ((*i as f32) / 65536.0).to_string() + "fill",
            Filll(i) =>
                ((*i as f32) / 65536.0).to_string() + "filll",
        }
    }
}

#[derive(Copy,Clone)]
pub struct MuSkip {
    pub base : i32,
    pub stretch : Option<MuSkipDim>,
    pub shrink: Option<MuSkipDim>
}
impl Display for MuSkip {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",self.to_string())
    }
}
impl Div<i32> for MuSkip {
    type Output = MuSkip;
    fn div(self, rhs: i32) -> Self::Output {
        MuSkip {
            base:round_f((self.base as f32) / (rhs as f32)),
            stretch:self.stretch,shrink:self.shrink
        }
    }
}
impl MuSkip {
    pub fn negate(self) -> MuSkip {
        MuSkip {
            base:-self.base,
            stretch:self.stretch.map(|x| x.negate()),
            shrink:self.shrink.map(|x| x.negate())
        }
    }
    pub fn to_string(&self) -> String {
        mudimtostr(self.base) + &match self.stretch {
            None => "".to_string(),
            Some(s) => " plus ".to_string() + &s.to_string()
        } + &match self.shrink {
            None => "".to_string(),
            Some(s) => " minus ".to_string() + &s.to_string()
        }
    }
}
impl Add<MuSkip> for MuSkip {
    type Output = MuSkip;
    fn add(self, rhs: MuSkip) -> Self::Output {
        MuSkip { base:self.base + rhs.base, stretch:self.stretch, shrink:self.shrink }
    }
}

#[derive(Copy,Clone)]
pub enum Numeric {
    Int(i32),
    Dim(i32),
    Float(f32),
    Skip(Skip),
    MuSkip(MuSkip)
}
impl Numeric {
    pub fn negate(self) -> Numeric {
        use Numeric::*;
        match self {
            Int(i) => Int(-i),
            Dim(i) => Dim(-i),
            Float(f) => Float(-f),
            Skip(sk) => Skip(sk.negate()),
            MuSkip(sk) => MuSkip(sk.negate())
        }
    }
}
pub fn numtostr(dim : i32,suff:&str) -> String {
    let mut ret = format!("{:.5}",round(dim)).to_string();
    loop {
        if ret.ends_with("0") {
            ret.pop();
        } else if ret.ends_with(".") {
            return ret + "0" + suff
        }
        else {
            return ret + suff
        }
    }
}
pub fn dimtostr(dim:i32) -> String { numtostr(dim,"pt") }
pub fn mudimtostr(dim:i32) -> String {
    numtostr(dim,"mu")
}
pub fn round(input : i32) -> f64 {
    let mut i = 1.0 as f64;
    loop {
        let rounded = (((input as f64) / 65536.0) * i).round() / i;
        if ((rounded * 65536.0).round() as i32) == input {
            return rounded
        } else {
            i = i * 10.0;
        }
    }
}
pub fn round_f(f:f32) -> i32 {
    f.round() as i32//floor() as i32//if f < 1.0 {0} else {f.round() as i32 }
}
impl Numeric {
    fn as_string(&self) -> String {
        use Numeric::*;
        match self {
            Int(i) => i.to_string(),
            Dim(i) => dimtostr(*i),
            Float(f) => f.to_string(),
            Skip(sk) => sk.to_string(),
            MuSkip(ms) => ms.to_string()
        }
    }
    pub fn as_int(&self) -> Numeric {
        use Numeric::*;
        match self {
            Int(_) => self.clone(),
            Dim(i) => Int(*i),
            Float(f) => Int(round_f(*f)),
            Skip(sk) => Int(sk.base),
            MuSkip(ms) => Int(ms.base)
        }
    }
    pub fn get_i32(&self) -> i32 {
        use Numeric::*;
        match self {
            Int(i) => *i,
            Dim(i) => *i,
            Float(f) => round_f(*f),
            Skip(sk) => sk.base,
            MuSkip(ms) => ms.base
        }
    }
}
impl std::ops::Div for Numeric {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        use Numeric::*;
        match (self,rhs) {
            (Int(i),Int(j)) => Int(((i as f32)/(j as f32)).round() as i32),
            (Int(i),Float(f)) => Int(round_f((i as f32)/f)),
            (Dim(i),Dim(f)) => Dim(round_f((i as f32)/((f as f32) / 65536.0))),
            (Dim(i),Skip(f)) => Dim(round_f((i as f32)/(f.base as f32 / 65536.0))),
            (Dim(i),Int(j)) => Dim(round_f((i as f32)/(j as f32))),
            _ => todo!("{}/{}",self,rhs)
        }
    }
}
impl std::ops::Mul for Numeric {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        use Numeric::*;
        match (self,rhs) {
            (Int(i),Int(j)) => Int(i*j),
            (Int(i),Float(j)) => Int(round_f((i as f32)*j)),
            (Float(i),Int(j)) => Float(i*(j as f32)),
            (Float(i),Float(j)) => Float(i*j),
            (Dim(i),Dim(f)) => Dim(round_f((i as f32) * (f as f32 / 65536.0))),
            (Dim(i),Skip(f)) => Dim(round_f((i as f32) * (f.base as f32 / 65536.0))),
            (Dim(i),Int(f)) => Dim(i * f),
            (Dim(i),Float(f)) => Dim(round_f((i as f32) * f)),
            _ => todo!("{}*{}",self,rhs)
        }
    }
}
impl std::ops::Add for Numeric {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        use Numeric::*;
        match (self,rhs) {
            (Int(i),Int(j)) => Int(i+j),
            (Dim(i),Int(j)) => Int(i+j),
            (Dim(i),Dim(j)) => Dim(i+j),
            (Int(i),Dim(j)) => Int(i+j),
            (Skip(sk),Dim(i)) => Skip(crate::interpreter::dimensions::Skip {
                base:sk.base + i,stretch:sk.stretch,shrink:sk.shrink
            }),
            (Skip(sk1),Skip(sk2)) => Skip(crate::interpreter::dimensions::Skip {
                base:sk1.base+sk2.base,stretch:sk1.stretch,shrink:sk1.shrink
            }),
            _ => todo!("{}+{}",self,rhs)
        }
    }
}

impl std::ops::Sub for Numeric {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        use Numeric::*;
        match (self,rhs) {
            (Int(i),Int(j)) => Int(i-j),
            (Dim(i),Int(j)) => Int(i-j),
            (Dim(i),Dim(j)) => Dim(i - j),
            (Skip(s),Skip(t)) => Skip(crate::interpreter::dimensions::Skip {
                base:s.base + t.base,stretch:s.stretch,shrink:s.shrink
            }),
            _ => todo!("{}-{}",self,rhs)
        }
    }
}
impl std::fmt::Display for Numeric {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",self.as_string())
    }
}
