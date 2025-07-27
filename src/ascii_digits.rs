use ratatui::text::Line;
use ratatui::style::{Color, Style};

pub fn get_digit_lines(digit: char) -> Vec<&'static str> {
    match digit {
        '0' => vec![
            " 0000 ",
            "00  00",
            "00  00", 
            "00  00",
            " 0000 ",
        ],
        '1' => vec![
            " 1111 ",
            "   11 ",
            "   11 ",
            "   11 ",
            "111111",
        ],
        '2' => vec![
            " 2222 ",
            "22  22",
            "   22 ",
            "  22  ",
            "222222",
        ],
        '3' => vec![
            " 3333 ",
            "33  33",
            "   333",
            "33  33",
            " 3333 ",
        ],
        '4' => vec![
            "44  44",
            "44  44",
            "444444",
            "    44",
            "    44",
        ],
        '5' => vec![
            "555555",
            "55    ",
            "55555 ",
            "    55",
            "55555 ",
        ],
        '6' => vec![
            " 6666 ",
            "66    ",
            "66666 ",
            "66  66",
            " 6666 ",
        ],
        '7' => vec![
            "777777",
            "   77 ",
            "  77  ",
            " 77   ",
            "77    ",
        ],
        '8' => vec![
            " 8888 ",
            "88  88",
            " 8888 ",
            "88  88",
            " 8888 ",
        ],
        '9' => vec![
            " 9999 ",
            "99  99",
            " 99999",
            "    99",
            " 9999 ",
        ],
        ':' => vec![
            "      ",
            "  ::  ",
            "      ",
            "  ::  ",
            "      ",
        ],
        _ => vec![
            "      ",
            "      ",
            "      ",
            "      ",
            "      ",
        ]
    }
}

pub fn create_time_display_lines(time_str: &str, color: Color) -> Vec<Line> {
    let chars: Vec<char> = time_str.chars().collect();
    let mut lines = vec![String::new(); 5]; // 5 lines for each digit
    
    // Build each line by concatenating the corresponding line from each digit
    for char in chars {
        let digit_lines = get_digit_lines(char);
        for (i, digit_line) in digit_lines.iter().enumerate() {
            if i < 5 {
                lines[i].push_str(digit_line);
                lines[i].push(' '); // Add space between digits
            }
        }
    }
    
    // Convert to ratatui Lines with color
    lines.into_iter()
        .map(|line| Line::from(line).style(Style::default().fg(color)))
        .collect()
}