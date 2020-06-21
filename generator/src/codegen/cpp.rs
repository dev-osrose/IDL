use super::Codegen;
use super::flat_ast::*;
use std::io::Result;
use ::heck::*;

pub struct Generator<'a> {
    output_dir: &'a std::path::Path,
    select: Select,
    hwriter: Option<::writer::Writer<std::fs::File>>,
    cwriter: Option<::writer::Writer<std::fs::File>>,
    version: &'a str,
    stem: &'a str
}

enum Select {
    H,
    CPP
}

impl<'a> Generator<'a> {
    pub fn new(output_dir: &'a std::path::Path, stem: &'a str, version: &'a str) -> Self {
        Self {
            output_dir,
            select: Select::H,
            hwriter: None,
            cwriter: None,
            version,
            stem
        }
    }

    fn convert_basic_types(type_: &str) -> Option<&'static str> {
        match type_ {
            "char" => Some("char"),
            "u8" => Some("uint8_t"),
            "i8" => Some("int8_t"),
            "u16" => Some("uint16_t"),
            "i16" => Some("int16_t"),
            "u32" => Some("uint32_t"),
            "i32" => Some("int32_t"),
            "u64" => Some("uint64_t"),
            "i64" => Some("int64_t"),
            "float" => Some("float"),
            "double" => Some("double"),
            "string" => Some("std::string"),
            _ => None
        }
    }

    fn select(&mut self, s: Select) {
        self.select = s;
    }

    fn indent(&mut self) {
        match self.select {
            Select::H => self.hwriter.as_mut().unwrap().indent(),
            Select::CPP => self.cwriter.as_mut().unwrap().indent()
        };
    }

    fn dedent(&mut self) {
        match self.select {
            Select::H => self.hwriter.as_mut().unwrap().dedent(),
            Select::CPP => self.cwriter.as_mut().unwrap().dedent()
        };
    }

    fn write(&mut self, val: impl AsRef<str>) -> Result<&mut Self> {
        match self.select {
            Select::H => self.hwriter.as_mut().unwrap().write(val)?,
            Select::CPP => self.cwriter.as_mut().unwrap().write(val)?
        };
        Ok(self)
    }

    fn generate_header(&mut self) -> Result<String> {
        let filename = self.output_dir.to_str().unwrap().to_owned() + &format!("{}.h", self.stem);
        let file = std::fs::File::create(filename.clone())?;
        self.hwriter = Some(::writer::Writer::new(file));
        Ok(filename)
    }

    fn generate_source(&mut self) -> Result<String> {
        let filename = self.output_dir.to_str().unwrap().to_owned() + &format!("{}.cpp", self.stem);
        let file = std::fs::File::create(filename.clone())?;
        self.cwriter = Some(::writer::Writer::new(file));
        Ok(filename)
    }

    fn get_type(elem: &Element) -> String {
        let type_ = if let Some(basic) = Generator::convert_basic_types(elem.type_()) {
            basic
        } else {
            elem.type_()
        };
        if let Some(occurs) = elem.occurs().as_ref() {
            match occurs {
                Occurs::Unbounded => format!("std::vector<{}>", type_),
                Occurs::Num(n) => format!("std::array<{}, {}>", type_, n)
            }
        } else {
            type_.to_string()
        }
    }

    fn write_choice_header(&mut self, name: &str, choice: &Choice) -> Result<()> {
        if let Some(doc) = choice.doc().as_ref() {
            cg!(self, "/* {} */", doc);
        }
        cg!(self, "class {} {{", name);
        self.indent();
        cg!(self, "public:");
        self.indent();
        for elem in choice.elements() {
            let type_ = Generator::get_type(elem);
            if let Some(doc) = elem.doc().as_ref() {
                cg!(self, "/* {} */", doc);
            }
            cg!(self, "const {}& get_{}() const noexcept;", type_, elem.name());
            cg!(self, "{0}& set_{1}(const {2}& {1});", name, elem.name(), type_);
            cg!(self, "{}& make_{}();", type_, elem.name());
            cg!(self);
        }
        cg!(self, "const std::string_view selection() const noexcept;");
        cg!(self);
        self.dedent();
        cg!(self, "private:");
        self.indent();
        let tt: Vec<_> = choice.elements().iter().map(|e| ", ".to_string() + &Generator::get_type(e)).collect();
        let types = String::from_utf8(tt.iter().map(|e| e.bytes()).flatten().collect()).unwrap();
        cg!(self, "std::variant<std::monostate{}> __data;", types);
        self.dedent();
        self.dedent();
        cg!(self, "}};");
        Ok(())
    }

    fn write_choice_source(&mut self, name: &str, choice: &Choice) -> Result<()> {
        for elem in choice.elements() {
            let type_ = Generator::get_type(elem);
            cg!(self, "const {}& {}::get_{}() const noexcept {{", type_, name, elem.name());
            self.indent();
            cg!(self, "return std::get<{}>(__data);", type_);
            self.dedent();
            cg!(self, "}}");
            cg!(self);
            cg!(self, "{0}& {0}::set_{1}(const {2}& {1}) {{", name, elem.name(), type_);
            self.indent();
            cg!(self, "__data = {};", elem.name());
            cg!(self, "return *this;");
            self.dedent();
            cg!(self, "}}");
            cg!(self);
            cg!(self, "{}& {}::make_{}() {{", type_, name, elem.name());
            self.indent();
            cg!(self, "{} tmp;", type_);
            cg!(self, "set_{}(tmp);", elem.name());
            cg!(self, "return std::get<{}>(__data);", type_);
            self.dedent();
            cg!(self, "}}");
            cg!(self);
        }
        cg!(self, "const std::string_view {}::selection() const noexcept {{", name);
        self.indent();
        cg!(self, "const size_t index = __data.index();");
        cg!(self, "switch (index) {{");
        self.indent();
        for (idx, elem) in choice.elements().iter().enumerate() {
            cg!(self, "case {}:", idx + 1); // because we have the monostate
            self.indent();
            cg!(self, "return \"{}\";", elem.name());
            self.dedent();
        }
        cg!(self, "default:");
        self.indent();
        cg!(self, "return \"unselected\";");
        self.dedent();
        self.dedent();
        cg!(self, "}}");
        self.dedent();
        cg!(self, "}}");
        cg!(self);
        Ok(())
    }

    fn write_choice(&mut self, name: &str, choice: &Choice) -> Result<()> {
        self.select(Select::H);
        self.write_choice_header(name, choice)?;
        self.select(Select::CPP);
        self.write_choice_source(name, choice)
    }

    fn write_sequence_header(&mut self, name: &str, s: &Sequence) -> Result<()> {
        if let Some(doc) = s.doc().as_ref() {
            cg!(self, "/* {} */", doc);
        }
        cg!(self, "class {} {{", name);
        self.indent();
        cg!(self, "public:");
        self.indent();
        for elem in s.elements() {
            let type_ = Generator::get_type(elem);
            cg!(self, "const {}& get_{}() const noexcept;", type_, elem.name());
            cg!(self, "{0}& set_{1}(const {2}& {1});", name, elem.name(), type_);
            cg!(self);
        }
        cg!(self);
        self.dedent();
        cg!(self, "private:");
        self.indent();
        for elem in s.elements() {
            let type_ = Generator::get_type(elem);
            if let Some(doc) = elem.doc().as_ref() {
                cg!(self, "/* {} */", doc);
            }
            cg!(self, "{} {};", type_, elem.name());
        }
        self.dedent();
        self.dedent();
        cg!(self, "}};");
        Ok(())
    }

    fn write_sequence_source(&mut self, name: &str, s: &Sequence) -> Result<()> {
        for elem in s.elements() {
            let type_ = Generator::get_type(elem);
            cg!(self, "const {}& {}::get_{}() const noexcept {{", type_, name, elem.name());
            self.indent();
            cg!(self, "return {};", elem.name());
            self.dedent();
            cg!(self, "}}");
            cg!(self);
            cg!(self, "{0}& {0}::set_{1}(const {2}& {1}) {{", name, elem.name(), type_);
            self.indent();
            cg!(self, "this->{0} = {0};", elem.name());
            cg!(self, "return *this;");
            self.dedent();
            cg!(self, "}}");
            cg!(self);
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn write_sequence(&mut self, name: &str, s: &Sequence) -> Result<()> {
        self.select(Select::H);
        self.write_sequence_header(name, s)?;
        self.select(Select::CPP);
        self.write_sequence_source(name, s)
    }

    fn write_restriction(&mut self, name: &str, r: &Restriction) -> Result<()> {
        if let Some(doc) = r.doc().as_ref() {
            cg!(self, "/* {} */", doc);
        }
        cg!(self, "enum class {} : uint16_t {{", name);
        self.indent();
        for elem in r.contents() {
            match elem {
                RestrictionContent::Enumeration(e) => {
                    if let Some(doc) = e.doc().as_ref() {
                        cg!(self, "/* {} */", doc);
                    }
                    cg!(self, "{} = {},", e.value().to_shouty_snake_case(), e.id());
                }
            }
        }
        self.dedent();
        cg!(self, "}};");
        Ok(())
    }

    fn write_simple_type_header(&mut self, s: &SimpleType) -> Result<()> {
        if let Some(doc) = s.doc() {
            cg!(self, "/* {} */", doc);
        }
        match s.content() {
            SimpleTypeContent::Restriction(r) => self.write_restriction(s.name(), r)?,
        }
        Ok(())
    }

    fn write_simple_type_source(&mut self, _: &SimpleType) -> Result<()> {
        Ok(())
    }

    fn write_simple_type(&mut self, s: &SimpleType) -> Result<()> {
        self.select(Select::H);
        self.write_simple_type_header(s)?;
        self.select(Select::CPP);
        self.write_simple_type_source(s)
    }

    fn write_complex_type_header(&mut self, c: &ComplexType) -> Result<()> {
        if let Some(doc) = c.doc() {
            cg!(self, "/* {} */", doc);
        }
        match c.content() {
            ComplexTypeContent::Choice(choice) => self.write_choice_header(c.name(), choice)?,
            ComplexTypeContent::Seq(s) => self.write_sequence_header(c.name(), s)?,
            ComplexTypeContent::Empty => {
                self.select(Select::H);
                cg!(self, "struct {} {{}};", c.name());
            }
        }
        Ok(())
    }

    fn write_complex_type_source(&mut self, c: &ComplexType) -> Result<()> {
        match c.content() {
            ComplexTypeContent::Choice(choice) => self.write_choice_source(c.name(), choice)?,
            ComplexTypeContent::Seq(s) => self.write_sequence_source(c.name(), s)?,
            ComplexTypeContent::Empty => {}
        }
        Ok(())
    }

    fn write_complex_type(&mut self, c: &ComplexType) -> Result<()> {
        self.select(Select::H);
        self.write_complex_type_header(c)?;
        self.select(Select::CPP);
        self.write_complex_type_source(c)
    }
}

impl<'a> Codegen for Generator<'a> {
    fn generate(&mut self, packet: &Packet) -> Result<Vec<String>> {
        let mut vec = Vec::new();
        vec.push(self.generate_header()?);
        vec.push(self.generate_source()?);

        self.select(Select::H);
        cg!(self, "/* Generated with IDL v{} */\n", self.version);
        cg!(self,"#include <vector>");
        cg!(self, "#include <string>");
        cg!(self, "#include <array>");
        cg!(self, "#include <variant>");
        cg!(self, "#include <string_view>");
        cg!(self);
        cg!(self, "namespace Packet {{");
        self.select(Select::CPP);
        cg!(self, "/* Generated with IDL v{} */\n", self.version);
        cg!(self,"#include \"{}\"", vec[0]);
        cg!(self);
        cg!(self, "namespace Packet {{");

        for type_ in packet.contents().iter().filter(|e| e.is_type()) {
            self.select(Select::H);
            cg!(self);
            self.select(Select::CPP);
            cg!(self);
            match type_ {
                PacketContent::Simple(s) => self.write_simple_type(s)?,
                PacketContent::Complex(c) => self.write_complex_type(c)?,
                _ => {}
            }
        }
        self.select(Select::H);
        cg!(self);
        self.select(Select::CPP);
        cg!(self);
        let mut choice = Choice::new(packet.doc().as_ref().cloned());
        for elem in packet.contents().iter().filter_map(|e| match e {
            PacketContent::Element(e) => Some(e),
            _ => None
        }) {
            choice.add_element(elem.clone());
        }
        self.write_choice("Packet", &choice)?;
        self.select(Select::H);
        cg!(self, "}} // namespace Packet");
        self.select(Select::CPP);
        cg!(self, "}} // namespace Packet");
        Ok(vec)
    }
}