use std;

use Error;
use StringExt;

pub struct ArrayIter<'a>{
    array_iter:std::iter::Filter<std::str::Split<'a,char>, fn(&& str) -> bool>,
    name:&'static str,
}

impl<'a> ArrayIter<'a> {
    pub fn new(text:&'a String, size:usize, name:&'static str) -> Self {
        fn is_not_empty(s:&&str) -> bool {
            *s!=""
        }

        ArrayIter {
            array_iter:text.split(' ').filter(is_not_empty),
            name:name,
        }
    }

    pub fn read_str(&mut self) -> Result<&str,Error> {
        match self.array_iter.next() {
            Some( v ) => Ok( v ),
            None => Err(Error::Other( format!("not all values of {} array have been read", self.name) )),
        }
    }

    pub fn read_f32(&mut self) -> Result<f32,Error> {
        let value_str=self.read_str()?;
        value_str.parse_as_f32("array element")
    }

    pub fn read_i32(&mut self) -> Result<i32,Error> {
        let value_str=self.read_str()?;
        value_str.parse_as_i32("array element")
    }

    pub fn read_usize(&mut self) -> Result<usize,Error> {
        let value_str=self.read_str()?;
        value_str.parse_as_usize("array element")
    }
}
