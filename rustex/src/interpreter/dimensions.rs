use std::fmt::{Display, Formatter};

pub fn pt(f:f64) -> f64 { f * 65536.0 }
pub fn inch(f:f64) -> f64 { pt(f) * 72.27 }
pub fn cm(f:f64) -> f64 { inch(f) / 2.54 }
pub fn mm(f:f64) -> f64 { cm(f) / 10.0 }

#[derive(Copy,Clone)]
pub enum SkipDim {
    Pt(i64),
    Fil(i64),
    Fill(i64),
    Filll(i64)
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
    pub base : i64,
    pub stretch : Option<SkipDim>,
    pub shrink: Option<SkipDim>
}
impl Display for Skip {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",self.to_string())
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


#[derive(Copy,Clone)]
pub enum MuSkipDim {
    Mu(i64),
    Fil(i64),
    Fill(i64),
    Filll(i64)
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
    pub base : i64,
    pub stretch : Option<MuSkipDim>,
    pub shrink: Option<MuSkipDim>
}
impl Display for MuSkip {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",self.to_string())
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

#[derive(Copy,Clone)]
pub enum Numeric {
    Int(i64),
    Dim(i64),
    Float(f64),
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
pub fn numtostr(dim : i64,suff:&str) -> String {
    let val = round(dim);
    if (val * 10.0).round() / 10.0 == val {
        format!("{:.1}{}",val,suff).to_string()
    } else if (val * 100.0).round() / 100.0 == val {
        format!("{:.2}{}",val,suff).to_string()
    } else if (val * 1000.0).round() / 1000.0 == val {
        format!("{:.3}{}",val,suff).to_string()
    } else if (val * 1000.0).round() / 1000.0 == val {
        format!("{:.4}{}",val,suff).to_string()
    } else if (val * 10000.0).round() / 10000.0 == val {
        format!("{:.5}{}",val,suff).to_string()
    } else if (val * 100000.0).round() / 100000.0 == val {
        format!("{:.6}{}",val,suff).to_string()
    } else if (val * 1000000.0).round() / 1000000.0 == val {
        format!("{:.7}{}",val,suff).to_string()
    } else {
        format!("{:.8}{}",val,suff).to_string()
    }
}
pub fn dimtostr(dim:i64) -> String { numtostr(dim,"pt") }
pub fn mudimtostr(dim:i64) -> String {
    numtostr(dim,"mu")
}
pub fn round(input : i64) -> f64 {
    let mut i = 1.0 as f64;
    let mut ip = ((input as f64) * 100000000.0).round() / 100000000.0;
    loop {
        let rounded = (((input as f64) / 65536.0) * i).round() / i;
        if ((rounded * 65536.0).round() as i64) == input {
            return rounded
        } else {
            i = i * 10.0;
        }
    }
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
            Float(f) => Int(f.round() as i64),
            Skip(sk) => Int(sk.base),
            MuSkip(ms) => Int(ms.base)
        }
    }
    pub fn get_i64(&self) -> i64 {
        use Numeric::*;
        match self {
            Int(i) => *i,
            Dim(i) => *i,
            Float(f) => f.round() as i64,
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
            (Int(i),Int(j)) => Int(((i as f64)/(j as f64)).round() as i64),
            (Int(i),Float(f)) => Int(((i as f64) / f).round() as i64),
            (Dim(i),Dim(f)) => Dim(((i as f64) / (f as f64 / 65536.0)).round() as i64),
            (Dim(i),Skip(f)) => Dim(((i as f64) / (f.base as f64 / 65536.0)).round() as i64),
            (Dim(i),Int(f)) => Dim(((i as f64) / (f as f64)).round() as i64),
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
            (Int(i),Float(j)) => Int(((i as f64)*j).round() as i64),
            (Float(i),Int(j)) => Float(i*(j as f64)),
            (Float(i),Float(j)) => Float(i*j),
            (Dim(i),Dim(f)) => Dim(((i as f64) * (f as f64 / 65536.0)).round() as i64),
            (Dim(i),Skip(f)) => Dim(((i as f64) * (f.base as f64 / 65536.0)).round() as i64),
            (Dim(i),Int(f)) => Dim(i * f),
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
            _ => todo!("{}-{}",self,rhs)
        }
    }
}
impl std::fmt::Display for Numeric {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",self.as_string())
    }
}
