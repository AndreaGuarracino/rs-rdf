use reader::lexer::rdf_lexer::{RdfLexer, TokensFromRdf};
use reader::lexer::token::Token;
use reader::input_reader::{InputReader, InputReaderHelper};
use std::io::Read;
use error::{Error, ErrorType};
use Result;

/// Produces tokens from NTriples input.
pub struct NTriplesLexer<R: Read> {
  input_reader: InputReader<R>,
  peeked_token: Option<Token>
}

/// Contains all implemented rules for creating tokens from NTriples syntax.
pub trait TokensFromNTriples<R: Read>: TokensFromRdf<R> {
  /// Parses the comment from the input and returns it as token.
  fn get_comment(mut input_reader: &mut InputReader<R>) -> Result<Token> {
    Self::consume_next_char(input_reader);    // consume '#'

    match input_reader.get_until_discard_leading_spaces(|c| c == '\n' || c == '\r') {
      Ok(chars) => {
        Self::consume_next_char(input_reader);  // consume comment delimiter
        Ok(Token::Comment(chars.to_string()))
      },
      Err(err) => {
        match err.error_type() {
          &ErrorType::EndOfInput(ref chars) => Ok(Token::Comment(chars.to_string())),
          _ => Err(Error::new(ErrorType::InvalidReaderInput,
                              "Invalid input while parsing comment."))
        }
      }
    }
  }

  /// Parses the language specification from the input and returns it as token.
  fn get_language_specification(input_reader: &mut InputReader<R>) -> Result<String> {
    match input_reader.get_until(InputReaderHelper::node_delimiter) {
      Ok(chars) => Ok(chars.to_string()),
      Err(err) => {
        match err.error_type() {
          &ErrorType::EndOfInput(ref chars) => Ok(chars.to_string()),
          _ => Err(Error::new(ErrorType::InvalidReaderInput,
                              "Invalid input for while parsing language specification."))
        }
      }
    }
  }

  /// Parses a literal from the input and returns it as token.
  fn get_literal(input_reader: &mut InputReader<R>) -> Result<Token> {
    Self::consume_next_char(input_reader);  // consume '"'
    let literal = input_reader.get_until(|c| c == '"')?.to_string();
    Self::consume_next_char(input_reader); // consume '"'

    match input_reader.peek_next_char()? {
      Some('@') => {
        Self::consume_next_char(input_reader); // consume '@'
        let language = Self::get_language_specification(input_reader)?;
        Ok(Token::LiteralWithLanguageSpecification(literal, language))
      },
      Some('^') => {
        Self::consume_next_char(input_reader); // consume '^'
        Self::consume_next_char(input_reader); // consume '^'

        match input_reader.peek_next_char()? {
          Some('<') => {    // data type is an URI (NTriples allows only URI data types)
            match Self::get_uri(input_reader)? {
              Token::Uri(datatype_uri) => {
                Ok(Token::LiteralWithUrlDatatype(literal, datatype_uri))
              },
              _ => Err(Error::new(ErrorType::InvalidReaderInput,
                                  "Invalid data type URI for literal."))
            }
          },
          Some(c) => Err(Error::new(ErrorType::InvalidReaderInput,
                                    "Invalid data type token: ". to_string() + &c.to_string())),
          None => Err(Error::new(ErrorType::InvalidReaderInput, "Invalid input."))
        }
      },
      _ => {
        Self::consume_next_char(input_reader); // consume '"'
        Ok(Token::Literal(literal))
      }
    }
  }

  /// Parses a URI from the input and returns it as token.
  fn get_uri(input_reader: &mut InputReader<R>) -> Result<Token> {
    Self::consume_next_char(input_reader);    // consume '<'
    let chars = input_reader.get_until(|c| c == '>')?;
    Self::consume_next_char(input_reader);    // consume '>'
    Ok(Token::Uri(chars.to_string()))
  }

  /// Parses a blank node ID from the input and returns it as token.
  fn get_blank_node(input_reader: &mut InputReader<R>) -> Result<Token> {
    Self::consume_next_char(input_reader);    // consume '_'

    // get colon after under score
    match input_reader.get_next_char()? {
      Some(':') => { }
      Some(c) => return Err(Error::new(ErrorType::InvalidReaderInput,
                                       "Invalid character while parsing blank node: ". to_string() + &c.to_string())),
      None => return Err(Error::new(ErrorType::InvalidReaderInput,
                                    "Error while parsing blank node."))
    }

    match input_reader.get_until(InputReaderHelper::node_delimiter) {
      Ok(chars) => Ok(Token::BlankNode(chars.to_string())),
      Err(err) => {
        match err.error_type() {
          &ErrorType::EndOfInput(ref chars) => Ok(Token::BlankNode(chars.to_string())),
          _ => Err(Error::new(ErrorType::InvalidReaderInput,
                              "Invalid input for lexer while parsing blank node."))
        }
      }
    }
  }
}

impl<R: Read> TokensFromRdf<R> for NTriplesLexer<R> { }
impl<R: Read> TokensFromNTriples<R> for NTriplesLexer<R> { }

impl<R: Read> RdfLexer<R> for NTriplesLexer<R> {
  /// Constructor for `NTriplesLexer`;
  ///
  /// # Examples
  ///
  /// ```
  /// use rdf_rs::reader::lexer::rdf_lexer::RdfLexer;
  /// use rdf_rs::reader::lexer::n_triples_lexer::NTriplesLexer;
  ///
  /// let input = "<example.org/a>".as_bytes();
  ///
  /// NTriplesLexer::new(input);
  /// ```
  fn new(input: R) -> NTriplesLexer<R> {
    NTriplesLexer {
      input_reader: InputReader::new(input),
      peeked_token: None
    }
  }

  /// Determines the next token from the input.
  ///
  /// # Examples
  ///
  /// ```
  /// use rdf_rs::reader::lexer::rdf_lexer::RdfLexer;
  /// use rdf_rs::reader::lexer::n_triples_lexer::NTriplesLexer;
  /// use rdf_rs::reader::lexer::token::Token;
  ///
  /// let input = "_:auto <example.org/b> \"test\" .".as_bytes();
  ///
  /// let mut lexer = NTriplesLexer::new(input);
  ///
  /// assert_eq!(lexer.get_next_token().unwrap(), Token::BlankNode("auto".to_string()));
  /// assert_eq!(lexer.get_next_token().unwrap(), Token::Uri("example.org/b".to_string()));
  /// assert_eq!(lexer.get_next_token().unwrap(), Token::Literal("test".to_string()));
  /// assert_eq!(lexer.get_next_token().unwrap(), Token::TripleDelimiter);
  /// ```
  ///
  /// # Failures
  ///
  /// - Input that does not conform to the NTriples standard.
  ///
  fn get_next_token(&mut self) -> Result<Token> {
    match self.peeked_token.clone() {
      Some(token) => {
        self.peeked_token = None;
        return Ok(token)
      },
      None => { }
    }

    match self.input_reader.peek_next_char_discard_leading_spaces()? {
      Some('#') => NTriplesLexer::get_comment(&mut self.input_reader),
      Some('"') => NTriplesLexer::get_literal(&mut self.input_reader),
      Some('<') => NTriplesLexer::get_uri(&mut self.input_reader),
      Some('_') => NTriplesLexer::get_blank_node(&mut self.input_reader),
      Some('.') => {
        NTriplesLexer::consume_next_char(&mut self.input_reader);  // consume '.'
        Ok(Token::TripleDelimiter)
      },
      None => Ok(Token::EndOfInput),
      Some(c) => Err(Error::new(ErrorType::InvalidReaderInput,
                                    "Invalid input: ".to_string() + &c.to_string()))
    }
  }

  /// Determines the next token without consuming it.
  ///
  /// # Examples
  ///
  /// ```
  /// use rdf_rs::reader::lexer::rdf_lexer::RdfLexer;
  /// use rdf_rs::reader::lexer::n_triples_lexer::NTriplesLexer;
  /// use rdf_rs::reader::lexer::token::Token;
  ///
  /// let input = "_:auto <example.org/b> \"test\" .".as_bytes();
  ///
  /// let mut lexer = NTriplesLexer::new(input);
  ///
  /// assert_eq!(lexer.peek_next_token().unwrap(), Token::BlankNode("auto".to_string()));
  /// assert_eq!(lexer.peek_next_token().unwrap(), Token::BlankNode("auto".to_string()));
  /// assert_eq!(lexer.get_next_token().unwrap(), Token::BlankNode("auto".to_string()));
  /// assert_eq!(lexer.peek_next_token().unwrap(), Token::Uri("example.org/b".to_string()));
  /// ```
  ///
  /// # Failures
  ///
  /// - End of input reached.
  /// - Invalid input that does not conform with NTriples standard.
  ///
  fn peek_next_token(&mut self) -> Result<Token> {
    match self.peeked_token.clone() {
      Some(token) => Ok(token),
      None => {
        let next = self.get_next_token()?;
        self.peeked_token = Some(next.clone());
        return Ok(next)
      }
    }
  }
}


#[cfg(test)]
mod tests {
  use reader::lexer::rdf_lexer::RdfLexer;
  use reader::lexer::n_triples_lexer::NTriplesLexer;
  use reader::lexer::token::Token;

  #[test]
  fn test_n_triples_parse_comment() {
    let input = "# Hello World!\n# Foo".as_bytes();

    let mut lexer = NTriplesLexer::new(input);

    assert_eq!(lexer.get_next_token().unwrap(), Token::Comment("Hello World!".to_string()));
    assert_eq!(lexer.get_next_token().unwrap(), Token::Comment("Foo".to_string()));
  }

  #[test]
  fn test_n_triples_parse_literal() {
    let input = "\"a\"".as_bytes();

    let mut lexer = NTriplesLexer::new(input);

    assert_eq!(lexer.get_next_token().unwrap(), Token::Literal("a".to_string()));
  }

  #[test]
  fn test_n_triples_parse_uri() {
    let input = "<example.org/a>".as_bytes();

    let mut lexer = NTriplesLexer::new(input);

    assert_eq!(lexer.get_next_token().unwrap(), Token::Uri("example.org/a".to_string()));
  }

  #[test]
  fn test_n_triples_parse_literal_with_language_specification() {
    let input = "\"a\"@abc".as_bytes();

    let mut lexer = NTriplesLexer::new(input);

    assert_eq!(lexer.get_next_token().unwrap(), Token::LiteralWithLanguageSpecification("a".to_string(),
                                                                                        "abc".to_string()));
  }

  #[test]
  fn test_n_triples_parse_blank_node() {
    let input = "_:auto".as_bytes();

    let mut lexer = NTriplesLexer::new(input);

    assert_eq!(lexer.get_next_token().unwrap(), Token::BlankNode("auto".to_string()));
  }

  #[test]
  fn test_n_triples_parse_literal_with_data_type() {
    let input = "\"a\"^^<example.org/abc>".as_bytes();

    let mut lexer = NTriplesLexer::new(input);

    assert_eq!(lexer.get_next_token().unwrap(), Token::LiteralWithUrlDatatype("a".to_string(),
                                                                              "example.org/abc".to_string()));
  }

  #[test]
  fn test_n_triples_parse_triple_delimiter() {
    let input = ".   \"a\"   .".as_bytes();

    let mut lexer = NTriplesLexer::new(input);

    assert_eq!(lexer.get_next_token().unwrap(), Token::TripleDelimiter);
    assert_eq!(lexer.get_next_token().unwrap(), Token::Literal("a".to_string()));
    assert_eq!(lexer.get_next_token().unwrap(), Token::TripleDelimiter);
  }
}