use std::io::{self, Write, stdout};

pub fn write<T, U>(out: &mut T, items: &[U], max_width: usize) -> io::Result<()>
where
    T: Write,
    U: AsRef<str>,
{
    let col_width = items
        .iter()
        .map(|item| item.as_ref().len() + 1)
        .max()
        .unwrap_or(0);
    if col_width == 0 {
        return Ok(());
    }
    let cols = max_width / col_width;
    for (i, item) in items.iter().enumerate() {
        write!(out, "{}", item.as_ref())?;
        for _ in 0..col_width - item.as_ref().len() {
            write!(out, " ")?;
        }
        if i % cols == cols - 1 {
            writeln!(out)?;
        }
    }
    Ok(())
}

pub fn print<T>(items: &[T])
where T: AsRef<str> {
    let cols = termion::terminal_size()
        .map(|(cols, _rows)| cols)
        .unwrap_or(80) as usize;
    write(&mut stdout(), items, cols).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_columned() {
        let mut buf = vec![];
        write(
            &mut buf,
            &[
                "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
            ],
            32,
        )
        .unwrap();
        assert_eq!(
            str::from_utf8(buf.as_slice()).unwrap(),
            "one   two   three four  five  \nsix   seven eight nine  ten   \n"
        );
    }
}
