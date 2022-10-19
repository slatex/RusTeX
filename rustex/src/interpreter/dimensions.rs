use std::fmt::{Display, Formatter};
use std::ops::{Add, Div};

pub fn pt(f:f64) -> f64 { f * 65536.0 }
pub fn inch(f:f64) -> f64 { pt(f) * 72.27 }
pub fn pc(f:f64) -> f64 { pt(f) * 12.0 }
pub fn cm(f:f64) -> f64 { inch(f) / 2.54 }
pub fn dd(f:f64) -> f64 {pt(f) / 1157.0 * 1238.0 }
pub fn cc(f:f64) -> f64 {dd(f) * 12.0 }
pub fn mm(f:f64) -> f64 { cm(f) / 10.0 }

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
                numtostr(*i,"pt"),
            Fil(i) =>
                numtostr(*i,"fil"),
            Fill(i) =>
                numtostr(*i,"fill"),
            Filll(i) =>
                numtostr(*i,"filll"),
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
            base:round_f((self.base as f64) / (rhs as f64)),
            stretch:self.stretch,shrink:self.shrink
        }
    }
}
impl std::ops::Mul<i32> for Skip {
    type Output = Skip;
    fn mul(self, rhs: i32) -> Self::Output {
        Skip {
            base:self.base * rhs,
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
                ((*i as f64) / 65536.0).to_string() + "mu",
            Fil(i) =>
                ((*i as f64) / 65536.0).to_string() + "fil",
            Fill(i) =>
                ((*i as f64) / 65536.0).to_string() + "fill",
            Filll(i) =>
                ((*i as f64) / 65536.0).to_string() + "filll",
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
            base:round_f((self.base as f64) / (rhs as f64)),
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
    Float(f64),
    Skip(Skip),
    BigInt(i64),
    MuSkip(MuSkip)
}
impl Numeric {
    pub fn negate(self) -> Numeric {
        use Numeric::*;
        match self {
            BigInt(i) => BigInt(-i),
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
pub fn round_f(f:f64) -> i32 {
    f.round() as i32//f.floor() as i32//if f < 1.0 {0} else {f.round() as i32 }
}
impl Numeric {
    fn as_string(&self) -> String {
        use Numeric::*;
        match self {
            BigInt(i) => i.to_string(),
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
            BigInt(i) => Int(*i as i32),
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
            BigInt(i) => *i as i32,
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
            (Int(i),BigInt(j)) => BigInt(((i as f64)/(j as f64)).round() as i64),
            (BigInt(i),Int(j)) => BigInt(((i as f64)/(j as f64)).round() as i64),
            (BigInt(i),BigInt(j)) => BigInt(((i as f64)/(j as f64)).round() as i64),
            (Int(i),Int(j)) => Int(((i as f64)/(j as f64)).round() as i32),
            (Int(i),Float(f)) => Int(round_f((i as f64)/f)),
            (Dim(i),Dim(f)) => Dim(round_f((i as f64)/((f as f64) / 65536.0))),
            (Dim(i),Skip(f)) => Dim(round_f((i as f64)/(f.base as f64 / 65536.0))),
            (Dim(i),Int(j)) => Dim(round_f((i as f64)/(j as f64))),
            (Dim(i),BigInt(j)) => Dim(round_f((i as f64)/(j as f64))),
            _ => todo!("{}/{}",self,rhs)
        }
    }
}
impl std::ops::Mul for Numeric {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        use Numeric::*;
        match (self,rhs) {
            (BigInt(i),Int(j)) => BigInt(i*(j as i64)),
            (BigInt(i),BigInt(j)) => BigInt(i*j),
            (Int(i),BigInt(j)) => BigInt((i as i64)*j),
            (Int(i),Int(j)) => Int(i*j),
            (Int(i),Float(j)) => Int(round_f((i as f64)*j)),
            (Float(i),Int(j)) => Float(i*(j as f64)),
            (Float(i),Float(j)) => Float(i*j),
            (Dim(i),Dim(f)) => Dim(round_f((i as f64) * (f as f64 / 65536.0))),
            (Dim(i),Skip(f)) => Dim(round_f((i as f64) * (f.base as f64 / 65536.0))),
            (Dim(i),Int(f)) => Dim(i * f),
            (Dim(i),BigInt(f)) => Dim(i * (f as i32)),
            (Dim(i),Float(f)) => Dim(round_f((i as f64) * f)),
            (Skip(s),Int(j)) => Skip(s * j),
            _ => todo!("{}*{}",self,rhs)
        }
    }
}
impl std::ops::Add for Numeric {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        use Numeric::*;
        match (self,rhs) {
            (Int(i),BigInt(j)) => BigInt((i as i64)+j),
            (BigInt(i),Int(j)) => BigInt((j as i64)+i),
            (BigInt(i),BigInt(j)) => BigInt(i+j),
            (Int(i),Int(j)) => Int(i+j),
            (Dim(i),Int(j)) => Int(i+j),
            (Dim(i),BigInt(j)) => BigInt((i as i64)+j),
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
            (BigInt(i),Int(j)) => BigInt(i-(j as i64)),
            (BigInt(i),BigInt(j)) => BigInt(i-j),
            (Int(i),BigInt(j)) => BigInt((i as i64)-j),
            (Int(i),Int(j)) => Int(i-j),
            (Dim(i),Int(j)) => Int(i-j),
            (Dim(i),BigInt(j)) => BigInt((i as i64)-j),
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
