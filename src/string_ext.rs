use Error;

pub trait StringExt {
    fn parse_as_f32(&self, name:&str) -> Result<f32,Error>;
    fn parse_as_i32(&self, name:&str) -> Result<i32,Error>;
    fn parse_as_usize(&self, name:&str) -> Result<usize,Error>;
}

impl StringExt for str {
    fn parse_as_f32(&self, name:&str) -> Result<f32,Error> {
        match self.parse::<f32>(){
            Ok ( f ) => Ok( f ),
            Err( _ ) => Err(Error::ParseFloatError( String::from(name), String::from(self)) ),
        }
    }

    fn parse_as_i32(&self, name:&str) -> Result<i32,Error> {
        match self.parse::<i32>(){
            Ok ( f ) => Ok( f ),
            Err( _ ) => Err(Error::ParseIntError( String::from(name), String::from(self)) ),
        }
    }

    fn parse_as_usize(&self, name:&str) -> Result<usize,Error> {
        match self.parse::<usize>(){
            Ok ( f ) => Ok( f ),
            Err( _ ) => Err(Error::ParseFloatError( String::from(name), String::from(self)) ),
        }
    }
}
