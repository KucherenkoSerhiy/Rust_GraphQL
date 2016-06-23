pub trait GraphQLReader {
    fn read(&mut self, value: string);
}

impl<T: Read> GraphQLReader for T {
    fn read(&mut self, value: string) {
        println!("Read the line: {}", value);
    }
}