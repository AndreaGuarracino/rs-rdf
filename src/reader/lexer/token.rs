
// todo
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
  Comment(String),
  Literal(String),
  LiteralWithUrlDatatype(String, String),   // first element is the literal, second the data type URL
  LiteralWithQNameDatatype(String, String), // first element is the literal, second the QName of the data type
  LiteralWithLanguageSpecification(String, String),
  Uri(String),
  BlankNode(String),
  TripleDelimiter,
  PrefixDirective(String, String),
  BaseDirective(String),
  QName(String, String),
  Prefix(String),
  PredicateListDelimiter,   // e.g. for Turtle syntax -> ;
  ObjectListDelimiter,      // e.g. for Turtle syntax -> ,
  EndOfInput,
}