use crate::commands::{PrimitiveExecutable, PrimitiveTeXCommand};

pub static PGFSYSDRIVER : PrimitiveExecutable = PrimitiveExecutable {
    expandable:true,
    name:"pgfsysdriver",
    _apply:|xp, int| {
        xp.2 = crate::interpreter::string_to_tokens("pgfsys-rust.def".into());
        Ok(())
    }
};

pub static COLORPUSH : PrimitiveExecutable = PrimitiveExecutable {
    expandable:false,
    name:"rustex!pgf!colorpush",
    _apply:|xp, int| {
        todo!()
    }
};

pub static COLORPOP : PrimitiveExecutable = PrimitiveExecutable {
    expandable:false,
    name:"rustex!pgf!colorpop",
    _apply:|xp, int| {
        todo!()
    }
};

pub static BEGINPICTURE : PrimitiveExecutable = PrimitiveExecutable {
    expandable:false,
    name:"rustex!pgf!beginpicture",
    _apply:|xp, int| {
        todo!()
    }
};

pub static ENDPICTURE : PrimitiveExecutable = PrimitiveExecutable {
    expandable:false,
    name:"rustex!pgf!endpicture",
    _apply:|xp, int| {
        todo!()
    }
};

pub static PGFHBOX : PrimitiveExecutable = PrimitiveExecutable {
    expandable:false,
    name:"rustex!pgf!hbox",
    _apply:|xp, int| {
        todo!()
    }
};

pub static TYPESETPICTUREBOX : PrimitiveExecutable = PrimitiveExecutable {
    expandable:false,
    name:"rustex!pgf!typesetpicturebox",
    _apply:|xp, int| {
        todo!()
    }
};

pub static PGFLITERAL : PrimitiveExecutable = PrimitiveExecutable {
    expandable:false,
    name:"rustex!pgf!literal",
    _apply:|xp, int| {
        todo!()
    }
};


// -------------------------------------------------------------------------------------------------

pub fn pgf_commands() -> Vec<PrimitiveTeXCommand> {vec![
    PrimitiveTeXCommand::Primitive(&PGFSYSDRIVER),
    PrimitiveTeXCommand::Primitive(&BEGINPICTURE),
    PrimitiveTeXCommand::Primitive(&ENDPICTURE),
    PrimitiveTeXCommand::Primitive(&PGFHBOX),
    PrimitiveTeXCommand::Primitive(&TYPESETPICTUREBOX),
    PrimitiveTeXCommand::Primitive(&PGFLITERAL),
    PrimitiveTeXCommand::Primitive(&COLORPUSH),
    PrimitiveTeXCommand::Primitive(&COLORPOP),
]}