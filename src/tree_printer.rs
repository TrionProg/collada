
pub struct TreePrinter{
    tabulation:String,
}

impl TreePrinter {
    pub fn new() -> Self {
        TreePrinter {
            tabulation:String::new(),
        }
    }

    pub fn print_tab(&self) {
        print!("{}",self.tabulation.as_str());
    }

    pub fn new_branch(&self, last:bool) -> TreePrinter {
        self.print_tab();

        if last {
            print!("└── ");
        }else{
            print!("├── ");
        }

        let mut tabulation=String::with_capacity(self.tabulation.len()+4);
        tabulation.push_str(self.tabulation.as_str());

        if last {
            tabulation.push_str("    ");
        }else{
            tabulation.push_str("│   ");
        }

        TreePrinter {
            tabulation:tabulation,
        }
    }
}
/*
impl<'a> Drop for TreePrinter<'a>{
    fn drop(&mut self) {
        if self.tabulation.len()>4 {
            let new_len=self.tabulation.len()-4;
            self.tabulation.truncate(new_len);
        }else{
            self.tabulation.clear();
        }
    }
}
*/
