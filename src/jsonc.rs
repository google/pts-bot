use std::io::{Read, Result};

enum State {
    Plain,
    Quote,
    Slash,
    Comment,
}

// Reader to filter out single line comments from the input stream
// to make the configuration compatible with standard JSON.
pub struct Reader<R: Read> {
    reader: R,
    state: State,
}

impl<R: Read> Reader<R> {
    pub fn new(reader: R) -> Self {
        Reader {
            reader,
            state: State::Plain,
        }
    }

    // Filter comments in input buffer. Returns the number of bytes
    // left in the buffer after filtering.
    fn filter(&mut self, buf: &mut [u8]) -> usize {
        let mut head = 0;

        fn write(buf: &mut [u8], head: &mut usize, c: u8) {
            buf[*head] = c;
            *head += 1;
        }

        use State::*;
        for cursor in 0..buf.len() {
            let c = buf[cursor];

            self.state = match self.state {
                Plain => match c as char {
                    '/' => Slash,
                    '"' => {
                        write(buf, &mut head, c);
                        Quote
                    }
                    _ => {
                        write(buf, &mut head, c);
                        Plain
                    }
                },
                Quote => {
                    write(buf, &mut head, c);
                    match c as char {
                        '"' => Plain,
                        _ => Quote,
                    }
                }
                Slash => match c as char {
                    '/' => Comment,
                    _ => {
                        // Normally would write back the '/' but this can cause
                        // an OOB if the reader is reading to [u8; 1]. Luckily
                        // standalone '/' characters are not expected in JSON
                        // so we can raise an early error here.
                        //
                        // write(buf, &mut head, '/' as u8);
                        // write(buf, &mut head, c);
                        // Plain
                        unreachable!("unexpected '/' in plain json text")
                    }
                },
                Comment => match c as char {
                    '\n' => {
                        write(buf, &mut head, c);
                        Plain
                    }
                    _ => Comment,
                },
            }
        }

        head
    }
}

impl<R: Read> Read for Reader<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut head = 0;
        loop {
            let len = self.reader.read(&mut buf[head..])?;
            if len == 0 {
                break;
            }

            head += self.filter(&mut buf[head..head + len]);
        }
        Ok(head)
    }
}
