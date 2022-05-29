use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Debug)]
pub struct SyntaxTree {
    pub statements: Vec<Statement>,
}

#[derive(Debug)]
pub enum Statement {
    Type(TypeData),
    Provider(ProviderData),
    Resource(ResourceData),
    Variable(VariableData),
}

#[derive(Clone, Debug)]
pub enum Expression {
    Literal(Literal),
    Reference(ReferencePath),
    Object(ObjectPath),
}

#[derive(Clone, Debug)]
pub struct ObjectPath {
    pub object: ReferencePath,
    pub member_path: Option<Vec<NameIdentifier>>,
}

#[derive(Clone, Debug)]
pub enum Literal {
    Boolean(bool),
    String(String),
    Number(Number),
}

#[derive(Clone, Debug)]
pub enum Number {
    Integer(String),
    FloatingPoint(String),
}

#[derive(Debug)]
pub struct ProviderData {
    pub provider_name: String,
}

#[derive(Debug)]
pub struct TypeData {
    pub type_name: String,
}

#[derive(Debug)]
pub struct VariableData {
    pub variable_name: String,
    pub type_name: ReferencePath,
    pub sensitive: bool,
    pub description: String,
}

#[derive(Debug)]
pub struct ResourceData {
    pub name: NameIdentifier,
    pub type_name: ReferencePath,
    pub body: Option<ResourceBody>,
    pub class: ResourceClass,
    pub is_scoped: bool,
}

impl ResourceData {
    pub fn new(
        name: NameIdentifier,
        type_name: ReferencePath,
        body: Option<ResourceBody>,
        class: ResourceClass,
        modifiers: &Vec<ResourceModifier>,
    ) -> ResourceData {
        ResourceData {
            name,
            type_name,
            body,
            class,
            is_scoped: modifiers.iter().any(|x| match x {
                ResourceModifier::Scoped => true,
            }),
        }
    }
}

pub type ResourceBody = HashMap<NameIdentifier, Expression>;

#[derive(Clone, Debug)]
pub struct ReferencePath {
    pub sequence: Vec<Reference>,
}

#[derive(Clone, Debug, Eq)]
pub enum Reference {
    Name(NameIdentifier),
    Super,
    Land,
}

#[derive(Clone, Debug, Eq)]
pub struct NameIdentifier {
    pub value: String,
}

impl Hash for Reference {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Name(s) => {
                state.write_u8(1);
                s.hash(state);
            }
            Self::Super => state.write_u8(2),
            Self::Land => state.write_u8(3),
        }
    }
}

impl PartialEq for Reference {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Name(s1), Self::Name(s2)) => s1 == s2,
            (Self::Super, Self::Super) => true,
            (Self::Land, Self::Land) => true,
            _ => false,
        }
    }
}

impl Hash for NameIdentifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl PartialEq for NameIdentifier {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

#[derive(Clone, Copy)]
pub enum ResourceModifier {
    Scoped,
}

#[derive(Debug)]
pub enum ResourceClass {
    Custom,
    Variable,
    Provider,
}
