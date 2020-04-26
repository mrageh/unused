use super::CtagItem;
use nom;
use std::collections::HashSet;
use std::default::Default;
use std::fmt::{Display, Formatter};
use std::fs;
use std::io;
use std::io::Error;

/// TagsReader provides a mechanism for attempting to read multiple ctags files until the first is
/// found
pub struct TagsReader<'a> {
    filenames: Vec<&'a str>,
}

/// A struct capturing possible failures when attempting to find and read tags files
pub enum ReadCtagsError {
    /// No tags file found
    ///
    /// This provides the paths attempted
    NoCtagsFile((Vec<String>, io::Error)),
    /// Incomplete parse; parsing was successful but didn't consume all input
    IncompleteParse,
    /// Parsing failed
    FailedParse(nom::Err<(String, nom::error::ErrorKind)>),
}

impl Display for ReadCtagsError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match *self {
            ReadCtagsError::NoCtagsFile((ref file_list, ref err)) => write!(
                f,
                "Unable to find ctags file (searched in {}): {}",
                file_list.join(", "),
                err
            ),
            ReadCtagsError::IncompleteParse => write!(f, "Unable to parse ctags file fully"),
            ReadCtagsError::FailedParse(ref err) => {
                write!(f, "Failed to parse ctags file: {}", err)
            }
        }
    }
}

impl<'a> Default for TagsReader<'a> {
    fn default() -> Self {
        TagsReader {
            filenames: vec![".git/tags", "tags", "tmp/tags"],
        }
    }
}

impl<'a> TagsReader<'a> {
    /// Loads and parses the first tags file it finds
    pub fn load(&self) -> Result<HashSet<CtagItem>, ReadCtagsError> {
        match self.read() {
            Ok(contents) => match CtagItem::parse(&contents) {
                Ok(("", outcome)) => Ok(outcome),
                Ok(_) => Err(ReadCtagsError::IncompleteParse),
                Err(e) => Err(ReadCtagsError::FailedParse(
                    e.map(|(v1, v2)| (v1.to_string(), v2)),
                )),
            },
            Err(e) => Err(ReadCtagsError::NoCtagsFile(e)),
        }
    }

    fn read(&self) -> Result<String, (Vec<String>, io::Error)> {
        Self::first_success(
            &self.filenames,
            Error::new(io::ErrorKind::Other, "No file provided"),
            Self::run,
        )
        .map_err(|e| (self.filenames.iter().map(|f| f.to_string()).collect(), e))
    }

    fn first_success<A, B, C, F>(values: &[A], default: C, f: F) -> Result<B, C>
    where
        A: Copy,
        F: Fn(A) -> Result<B, C>,
    {
        let mut outcome = Err(default);
        for &x in values.iter() {
            outcome = f(x);
            if outcome.is_ok() {
                break;
            }
        }
        outcome
    }

    fn run(filename: &str) -> Result<String, io::Error> {
        let contents = fs::read_to_string(filename)?;

        Ok(contents)
    }
}