// PICO-8 Lua dialect → standard Lua 5.2 preprocessor.
// Faithful port of preprocessor.zig.

pub fn preprocess(source: &str) -> String {
    let mut out: Vec<u8> = Vec::with_capacity(source.len());
    let mut in_long_comment = false;
    let mut long_comment_level: usize = 0;
    let mut in_long_string = false;
    let mut long_string_level: usize = 0;

    let mut first_line = true;
    for raw_line in source.as_bytes().split(|&b| b == b'\n') {
        if !first_line {
            out.push(b'\n');
        }
        first_line = false;

        if in_long_comment {
            if let Some(end_pos) = find_long_close(raw_line, long_comment_level) {
                in_long_comment = false;
                if end_pos < raw_line.len() {
                    preprocess_and_process_line(
                        &raw_line[end_pos..],
                        &mut out,
                        &mut in_long_comment,
                        &mut long_comment_level,
                        &mut in_long_string,
                        &mut long_string_level,
                    );
                }
            }
            continue;
        }

        if in_long_string {
            if let Some(end_pos) = find_long_close(raw_line, long_string_level) {
                out.extend_from_slice(&raw_line[..end_pos]);
                in_long_string = false;
                if end_pos < raw_line.len() {
                    preprocess_and_process_line(
                        &raw_line[end_pos..],
                        &mut out,
                        &mut in_long_comment,
                        &mut long_comment_level,
                        &mut in_long_string,
                        &mut long_string_level,
                    );
                }
            } else {
                out.extend_from_slice(raw_line);
            }
            continue;
        }

        preprocess_and_process_line(
            raw_line,
            &mut out,
            &mut in_long_comment,
            &mut long_comment_level,
            &mut in_long_string,
            &mut long_string_level,
        );
    }

    let final_bytes = insert_number_spaces(&out);
    String::from_utf8_lossy(&final_bytes).into_owned()
}

fn preprocess_and_process_line(
    raw_line: &[u8],
    out: &mut Vec<u8>,
    in_long_comment: &mut bool,
    long_comment_level: &mut usize,
    in_long_string: &mut bool,
    long_string_level: &mut usize,
) {
    let spaced = insert_number_spaces(raw_line);
    let expanded = expand_short_ifs(&spaced);
    let line: &[u8] = expanded.as_deref().unwrap_or(&spaced);
    process_line(
        line,
        out,
        in_long_comment,
        long_comment_level,
        in_long_string,
        long_string_level,
    );
}

fn insert_number_spaces(source: &[u8]) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::with_capacity(source.len());
    let mut i = 0;
    let mut in_str: u8 = 0;
    let mut in_long_str = false;
    let mut long_str_level: usize = 0;

    while i < source.len() {
        let ch = source[i];

        if in_long_str {
            if ch == b']' && match_long_close(source, i, long_str_level) {
                let close_len = long_str_level + 2;
                result.extend_from_slice(&source[i..i + close_len]);
                i += close_len;
                in_long_str = false;
            } else {
                result.push(ch);
                i += 1;
            }
            continue;
        }

        if in_str != 0 {
            if ch == b'\\' && i + 1 < source.len() {
                result.push(ch);
                i += 1;
                result.push(source[i]);
                i += 1;
                continue;
            }
            if ch == in_str {
                in_str = 0;
            }
            result.push(ch);
            i += 1;
            continue;
        }
        if ch == b'"' || ch == b'\'' {
            in_str = ch;
            result.push(ch);
            i += 1;
            continue;
        }
        if ch == b'[' {
            if let Some(level) = match_long_open(source, i) {
                in_long_str = true;
                long_str_level = level;
                let open_len = level + 2;
                result.extend_from_slice(&source[i..i + open_len]);
                i += open_len;
                continue;
            }
        }
        if ch == b'-' && i + 1 < source.len() && source[i + 1] == b'-' {
            let line_end = source[i..]
                .iter()
                .position(|&b| b == b'\n')
                .unwrap_or(source.len() - i);
            result.extend_from_slice(&source[i..i + line_end]);
            i += line_end;
            continue;
        }

        let is_num_start = is_number_start(source, i);
        if is_num_start {
            let mut end = i;
            if ch == b'0'
                && end + 1 < source.len()
                && (source[end + 1] == b'x' || source[end + 1] == b'X')
            {
                end += 2;
                while end < source.len() && (is_hex(source[end]) || source[end] == b'.') {
                    end += 1;
                }
            } else if ch == b'0'
                && end + 1 < source.len()
                && (source[end + 1] == b'b' || source[end + 1] == b'B')
            {
                end += 2;
                while end < source.len()
                    && (source[end] == b'0' || source[end] == b'1' || source[end] == b'.')
                {
                    end += 1;
                }
            } else {
                while end < source.len() && (source[end].is_ascii_digit() || source[end] == b'.') {
                    end += 1;
                }
            }
            result.extend_from_slice(&source[i..end]);
            // High-byte P8SCII glyphs are identifier characters in PICO-8
            // (they expand to synthetic `_p8_XX` names later), so a number
            // glued to one (`-2<glyph>.y-=8`, redash-7.p8.png) needs the
            // same separating space as a number glued to an ASCII letter --
            // otherwise the digit fuses into the glyph's expanded
            // identifier downstream.
            if end < source.len()
                && (source[end].is_ascii_alphabetic() || source[end] == b'_' || source[end] >= 0x80)
            {
                result.push(b' ');
            }
            i = end;
            continue;
        }

        if (ch == b')' || ch == b']') && i + 1 < source.len() {
            result.push(ch);
            i += 1;
            let next = source[i];
            if next.is_ascii_alphabetic() || next == b'_' || next >= 0x80 {
                result.push(b' ');
            }
            continue;
        }

        result.push(ch);
        i += 1;
    }
    result
}

fn is_number_start(line: &[u8], i: usize) -> bool {
    let ch = line[i];
    if ch.is_ascii_digit() {
        return i == 0
            || (!line[i - 1].is_ascii_alphanumeric()
                && line[i - 1] != b'_'
                && line[i - 1] != b'.');
    }
    if ch == b'.' && i + 1 < line.len() && line[i + 1].is_ascii_digit() {
        return i == 0
            || (!line[i - 1].is_ascii_digit() && line[i - 1] != b'.' && !is_hex(line[i - 1]));
    }
    false
}

fn is_hex(b: u8) -> bool {
    b.is_ascii_digit() || (b'a'..=b'f').contains(&b) || (b'A'..=b'F').contains(&b)
}

fn process_line(
    line: &[u8],
    out: &mut Vec<u8>,
    in_long_comment: &mut bool,
    long_comment_level: &mut usize,
    in_long_string: &mut bool,
    long_string_level: &mut usize,
) {
    let mut i = 0;
    let mut in_string: u8 = 0;
    // `?` is PICO-8's print() shorthand. It's valid wherever a statement can
    // start — not just at the start of a line; size-golfed carts glue it
    // directly after `end`/`then`/`)` etc with no separator — and, confirmed
    // against official PICO-8, its argument list always runs to the end of
    // the physical line (comments excepted, since those are stripped
    // lexically regardless of context). Anything else meant to follow on
    // the same line — `end`, `;`, more statements — gets swallowed too and
    // fails to compile in official PICO-8 as well; carts relying on that
    // are simply broken there too, not a pico-r gap.
    let mut print_shorthand_active = false;

    while i < line.len() {
        let ch = line[i];

        if *in_long_string {
            if ch == b']' && match_long_close(line, i, *long_string_level) {
                let close_len = *long_string_level + 2;
                out.extend_from_slice(&line[i..i + close_len]);
                i += close_len;
                *in_long_string = false;
            } else {
                out.push(ch);
                i += 1;
            }
            continue;
        }

        if in_string != 0 {
            if ch == b'\\' {
                i += 1;
                if i < line.len() {
                    let next = line[i];
                    if let Some(escape) = p8scii_control_escape(next) {
                        // PICO-8 control-code shorthand: `\^`,`\#`,`\-`,`\|`,`\+`
                        // each collapse to a single P8SCII control byte; the
                        // following character is NOT consumed as a parameter —
                        // it stays as ordinary string content, interpreted at
                        // draw_text() time (see gfx.rs's 0x01-0x06 handling).
                        out.extend_from_slice(escape);
                    } else if is_valid_lua52_escape(next) {
                        out.push(b'\\');
                        out.push(next);
                    } else {
                        out.extend_from_slice(b"\\092");
                        out.push(next);
                    }
                    i += 1;
                } else {
                    out.push(b'\\');
                }
                continue;
            }
            if ch == in_string {
                in_string = 0;
            }
            out.push(ch);
            i += 1;
            continue;
        }

        if print_shorthand_active
            && i + 1 < line.len()
            && ((ch == b'-' && line[i + 1] == b'-') || (ch == b'/' && line[i + 1] == b'/'))
        {
            out.push(b')');
            print_shorthand_active = false;
        }

        if ch == b'?' {
            // Size-golfed carts glue `?` directly after a keyword like
            // `then`/`else`/`do` with no separator (`then?"x"`). Since both
            // sides are identifier characters, splicing in `print(` verbatim
            // would fuse them into a single token (`thenprint`) for the real
            // Lua lexer -- insert a space when the preceding byte demands it.
            if out
                .last()
                .is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_')
            {
                out.push(b' ');
            }
            out.extend_from_slice(b"print(");
            print_shorthand_active = true;
            i += 1;
            continue;
        }

        if ch == b'-' && i + 1 < line.len() && line[i + 1] == b'-' {
            if i + 2 < line.len() && line[i + 2] == b'[' {
                if let Some(level) = match_long_open(line, i + 2) {
                    let content_start = i + 2 + level + 2;
                    if let Some(close_off) = find_long_close(&line[content_start..], level) {
                        i = content_start + close_off;
                        continue;
                    } else {
                        *in_long_comment = true;
                        *long_comment_level = level;
                        return;
                    }
                }
            }
            out.extend_from_slice(&line[i..]);
            return;
        }

        // PICO-8 also accepts C-style `//` line comments (confirmed via
        // oracle) -- rewrite to `--` so the real Lua lexer accepts it.
        if ch == b'/' && i + 1 < line.len() && line[i + 1] == b'/' {
            out.extend_from_slice(b"--");
            out.extend_from_slice(&line[i + 2..]);
            return;
        }

        if ch == b'"' || ch == b'\'' {
            in_string = ch;
            out.push(ch);
            i += 1;
            continue;
        }

        if ch == b'[' {
            if let Some(level) = match_long_open(line, i) {
                *in_long_string = true;
                *long_string_level = level;
                let open_len = level + 2;
                out.extend_from_slice(&line[i..i + open_len]);
                i += open_len;
                continue;
            }
        }

        if ch >= 0x80 {
            if let Some(s) = p8scii_button_id(ch) {
                out.extend_from_slice(s);
            } else {
                // PICO-8 treats high-byte glyphs as identifier characters; map
                // each byte deterministically to a Lua-safe identifier so code
                // like `fills = {A,B,...}` (glyph variable names) parses.
                let hex = b"0123456789abcdef";
                out.extend_from_slice(b"_p8_");
                out.push(hex[(ch >> 4) as usize]);
                out.push(hex[(ch & 0x0f) as usize]);
            }
            i += 1;
            continue;
        }

        if ch == b'!' && i + 1 < line.len() && line[i + 1] == b'=' {
            out.extend_from_slice(b"~=");
            i += 2;
            continue;
        }

        // 3-char bitwise: >>>, <<>, ><
        if i + 2 < line.len() {
            if ch == b'>' && line[i + 1] == b'>' && line[i + 2] == b'>' {
                if let Some(new_i) = try_bitwise_op(line, i, 3, b"lshr", out) {
                    i = new_i;
                    continue;
                }
            } else if ch == b'<' && line[i + 1] == b'<' && line[i + 2] == b'>' {
                if let Some(new_i) = try_bitwise_op(line, i, 3, b"rotl", out) {
                    i = new_i;
                    continue;
                }
            } else if ch == b'>' && line[i + 1] == b'>' && line[i + 2] == b'<' {
                if let Some(new_i) = try_bitwise_op(line, i, 3, b"rotr", out) {
                    i = new_i;
                    continue;
                }
            }
        }
        if i + 1 < line.len() {
            if ch == b'>'
                && line[i + 1] == b'>'
                && !(i + 2 < line.len() && (line[i + 2] == b'>' || line[i + 2] == b'<'))
            {
                if let Some(new_i) = try_bitwise_op(line, i, 2, b"shr", out) {
                    i = new_i;
                    continue;
                }
            } else if ch == b'<'
                && line[i + 1] == b'<'
                && !(i + 2 < line.len() && line[i + 2] == b'>')
            {
                if let Some(new_i) = try_bitwise_op(line, i, 2, b"shl", out) {
                    i = new_i;
                    continue;
                }
            } else if ch == b'^' && line[i + 1] == b'^' {
                if let Some(new_i) = try_bitwise_op(line, i, 2, b"bxor", out) {
                    i = new_i;
                    continue;
                }
            }
        }

        // Binary literals 0b...
        if ch == b'0'
            && i + 1 < line.len()
            && (line[i + 1] == b'b' || line[i + 1] == b'B')
            && (i == 0 || !line[i - 1].is_ascii_alphanumeric())
        {
            let mut end = i + 2;
            while end < line.len() && (line[end] == b'0' || line[end] == b'1' || line[end] == b'.')
            {
                end += 1;
            }
            if end > i + 2 {
                let val = parse_binary_literal(&line[i + 2..end]);
                let s = format!("{}", val);
                out.extend_from_slice(s.as_bytes());
                i = end;
                continue;
            }
        }

        // Compound assignments
        if let Some(new_i) = try_compound_assign(line, i, out) {
            i = new_i;
            continue;
        }

        // Integer division: \
        if ch == b'\\' && i + 1 < line.len() && line[i + 1] != b'=' {
            if let Some(new_i) = try_int_div(line, i, out) {
                i = new_i;
                continue;
            }
        }

        // Peek shortcuts
        if ch == b'@' || ch == b'$' || (ch == b'%' && !is_prev_value(line, i)) {
            if let Some(new_i) = try_peek_shortcut(line, i, out) {
                i = new_i;
                continue;
            }
        }

        // Single-char bitwise
        if ch == b'&' && !(i + 1 < line.len() && line[i + 1] == b'=') {
            if let Some(new_i) = try_bitwise_op(line, i, 1, b"band", out) {
                i = new_i;
                continue;
            }
        }
        if ch == b'|' && !(i + 1 < line.len() && line[i + 1] == b'=') {
            if let Some(new_i) = try_bitwise_op(line, i, 1, b"bor", out) {
                i = new_i;
                continue;
            }
        }

        // `~` (when not `~=`): binary XOR when a value precedes it
        // (`j~i` == `j^^i`, Lua-5.3 style -- confirmed via oracle:
        // 5~3 == 6; real corpus carts brandgnume-1.p8.png,
        // kokoroko-3.p8.png), otherwise unary bitwise NOT (`~x` ->
        // bnot(x)).
        if ch == b'~' && !(i + 1 < line.len() && line[i + 1] == b'=') {
            if is_prev_value(line, i) {
                if let Some(new_i) = try_bitwise_op(line, i, 1, b"bxor", out) {
                    i = new_i;
                    continue;
                }
            }
            let info = extract_simple_expr(line, i + 1);
            if !info.expr.is_empty() {
                out.extend_from_slice(b"bnot(");
                out.extend_from_slice(info.expr);
                out.push(b')');
                i = info.end;
                continue;
            }
        }

        // Number literal - emit with optional space after
        if is_number_start(line, i) {
            let mut end = i;
            if ch == b'0'
                && end + 1 < line.len()
                && (line[end + 1] == b'x' || line[end + 1] == b'X')
            {
                end += 2;
                while end < line.len() && (is_hex(line[end]) || line[end] == b'.') {
                    end += 1;
                }
            } else {
                while end < line.len() && (line[end].is_ascii_digit() || line[end] == b'.') {
                    end += 1;
                }
            }
            out.extend_from_slice(&line[i..end]);
            if end < line.len() && (line[end].is_ascii_alphabetic() || line[end] == b'_') {
                out.push(b' ');
            }
            i = end;
            continue;
        }

        out.push(ch);
        i += 1;
    }

    if print_shorthand_active {
        out.push(b')');
    }
}

fn is_prev_value(line: &[u8], pos: usize) -> bool {
    // Lua is whitespace-insensitive between tokens, so `timer %2` is
    // modulo exactly like `timer%2` -- skip spaces/tabs backward before
    // classifying, or the space made `%` look like the peek2-shortcut
    // (`timer peek2(2)`). Confirmed on a real corpus cart
    // (deepening-0.p8.png: `if timer %2==0 then`).
    let mut p = pos;
    while p > 0 && (line[p - 1] == b' ' || line[p - 1] == b'\t') {
        p -= 1;
    }
    if p == 0 {
        return false;
    }
    let prev = line[p - 1];
    // A keyword is not a value even though it ends in identifier
    // characters -- `return %addr` must classify `%` as the peek2
    // shortcut, not modulo on the "value" `return`. (`true`/`false`/`nil`
    // ARE value keywords and stay value-like.) Confirmed on a real corpus
    // cart (pizza_panda-1.p8.png: `then return %addr,...`).
    if prev.is_ascii_alphanumeric() || prev == b'_' {
        let mut ws = p;
        while ws > 0 && (line[ws - 1].is_ascii_alphanumeric() || line[ws - 1] == b'_') {
            ws -= 1;
        }
        const NONVALUE_KEYWORDS: &[&[u8]] = &[
            b"and",
            b"break",
            b"do",
            b"else",
            b"elseif",
            b"end",
            b"for",
            b"function",
            b"goto",
            b"if",
            b"in",
            b"local",
            b"not",
            b"or",
            b"repeat",
            b"return",
            b"then",
            b"until",
            b"while",
        ];
        if NONVALUE_KEYWORDS.contains(&&line[ws..p]) {
            return false;
        }
        return true;
    }
    // A high-byte P8SCII glyph expands to a synthetic `_p8_XX` identifier
    // (see the `ch >= 0x80` handling in `process_line`), so it counts as
    // value-like context too -- otherwise a glyph directly followed by `%`
    // was misclassified as the peek2-shortcut instead of modulo (the raw
    // byte isn't ASCII-alphanumeric, even though the identifier it expands
    // to clearly is a value). Confirmed against a real corpus cart
    // (batazubipe-0.p8.png: a glyph directly followed by `%40`).
    prev == b')' || prev == b']' || prev >= 0x80
}

fn parse_binary_literal(s: &[u8]) -> f64 {
    let mut int_part: u32 = 0;
    let mut frac_part: f64 = 0.0;
    let mut in_frac = false;
    let mut frac_denom: f64 = 1.0;
    for &ch in s {
        if ch == b'.' {
            in_frac = true;
            continue;
        }
        if in_frac {
            frac_denom *= 2.0;
            if ch == b'1' {
                frac_part += 1.0 / frac_denom;
            }
        } else {
            int_part = (int_part << 1) | (if ch == b'1' { 1 } else { 0 });
        }
    }
    int_part as f64 + frac_part
}

fn is_valid_lua52_escape(ch: u8) -> bool {
    matches!(
        ch,
        b'a' | b'b'
            | b'f'
            | b'n'
            | b'r'
            | b't'
            | b'v'
            | b'\\'
            | b'"'
            | b'\''
            | b'['
            | b']'
            | b'z'
            | b'x'
    ) || ch.is_ascii_digit()
}

fn p8scii_button_id(ch: u8) -> Option<&'static [u8]> {
    Some(match ch {
        0x83 => b"3",
        0x8B => b"0",
        0x8E => b"5",
        0x91 => b"1",
        0x94 => b"2",
        0x97 => b"4",
        _ => return None,
    })
}

// PICO-8 P8SCII control-code escape shorthand, confirmed against official
// PICO-8 (`ord(sub(s,i,i))` on each escaped string): `\X` collapses to a
// single control byte, emitted here as a zero-padded `\NNN` decimal escape
// so the real Lua lexer decodes it back to that one byte.
fn p8scii_control_escape(ch: u8) -> Option<&'static [u8]> {
    Some(match ch {
        b'*' => b"\\001",
        b'#' => b"\\002",
        b'-' => b"\\003",
        b'|' => b"\\004",
        b'+' => b"\\005",
        b'^' => b"\\006",
        _ => return None,
    })
}

fn match_keyword_at(line: &[u8], pos: usize, keyword: &[u8]) -> bool {
    if pos + keyword.len() > line.len() {
        return false;
    }
    if &line[pos..pos + keyword.len()] != keyword {
        return false;
    }
    if pos > 0 && (line[pos - 1].is_ascii_alphanumeric() || line[pos - 1] == b'_') {
        return false;
    }
    if pos + keyword.len() < line.len()
        && (line[pos + keyword.len()].is_ascii_alphanumeric() || line[pos + keyword.len()] == b'_')
    {
        return false;
    }
    true
}

// Scans a condition expression (starting right at its `if`/`while` — the
// position just after the keyword itself, before any leading whitespace)
// to determine whether it already has its own explicit `then`/`do`
// separator later on this same line, returning the index just past it if
// so. Tracks paren/bracket/brace depth (so operators/indexing after an
// initial parenthesized prefix don't get mistaken for the separator
// position) and a nesting depth of `if`/`while`/`for` keywords seen at
// depth 0, closed by `end` -- NOT by their own `then`/`do`, since a single
// nested `if...elseif...elseif...end` can contain several `then`s (one per
// clause) that all still belong to that one nested statement until its
// `end`. `then`/`do` only counts as THIS statement's own separator when
// found at nesting depth 0. See `expand_short_ifs`'s call site for the two
// real corpus patterns this distinguishes.
fn find_own_separator(line: &[u8], start: usize) -> Option<usize> {
    let mut i = start;
    let mut depth: i32 = 0;
    let mut in_str: u8 = 0;
    let mut nest: i32 = 0;
    while i < line.len() {
        let ch = line[i];
        if in_str != 0 {
            if ch == b'\\' && i + 1 < line.len() {
                i += 2;
                continue;
            }
            if ch == in_str {
                in_str = 0;
            }
            i += 1;
            continue;
        }
        if ch == b'"' || ch == b'\'' {
            in_str = ch;
            i += 1;
            continue;
        }
        if i + 1 < line.len()
            && ((ch == b'-' && line[i + 1] == b'-') || (ch == b'/' && line[i + 1] == b'/'))
        {
            return None;
        }
        if matches!(ch, b'(' | b'[' | b'{') {
            depth += 1;
            i += 1;
            continue;
        }
        if matches!(ch, b')' | b']' | b'}') {
            depth -= 1;
            i += 1;
            continue;
        }
        if depth == 0 && ch.is_ascii_alphabetic() {
            if match_keyword_at(line, i, b"if")
                || match_keyword_at(line, i, b"while")
                || match_keyword_at(line, i, b"for")
            {
                nest += 1;
            } else if match_keyword_at(line, i, b"end") {
                nest -= 1;
            } else if nest == 0
                && (match_keyword_at(line, i, b"then") || match_keyword_at(line, i, b"do"))
            {
                let kw_len = if line[i] == b't' { 4 } else { 2 };
                return Some(i + kw_len);
            }
            while i < line.len() && (line[i].is_ascii_alphanumeric() || line[i] == b'_') {
                i += 1;
            }
            continue;
        }
        i += 1;
    }
    None
}

fn expand_short_ifs(line: &[u8]) -> Option<Vec<u8>> {
    if !contains(line, b"if") && !contains(line, b"while") {
        return None;
    }

    let mut result: Vec<u8> = Vec::with_capacity(line.len() + 16);
    let mut ends_needed: usize = 0;
    let mut expanded_any = false;
    let mut i = 0;
    let mut in_str: u8 = 0;

    while i < line.len() {
        let ch = line[i];

        if in_str != 0 {
            if ch == b'\\' && i + 1 < line.len() {
                result.push(ch);
                i += 1;
                result.push(line[i]);
                i += 1;
                continue;
            }
            if ch == in_str {
                in_str = 0;
            }
            result.push(ch);
            i += 1;
            continue;
        }
        if ch == b'"' || ch == b'\'' {
            in_str = ch;
            result.push(ch);
            i += 1;
            continue;
        }

        if i + 1 < line.len()
            && ((ch == b'-' && line[i + 1] == b'-') || (ch == b'/' && line[i + 1] == b'/'))
        {
            // A `--` (or PICO-8's C-style `//`) comment runs to end of
            // line, so any still-pending synthetic `end`s must be emitted
            // BEFORE it -- appending them after the loop (the normal path)
            // would bury them inside the comment text. Confirmed on a real
            // corpus cart (dinkykong-0.p8.png: `if(not cstg.hr)return true
            // --todo:...`). The `//` form is copied verbatim here;
            // process_line rewrites it to `--` afterward.
            for _ in 0..ends_needed {
                result.extend_from_slice(b" end");
            }
            expanded_any |= ends_needed > 0;
            ends_needed = 0;
            result.extend_from_slice(&line[i..]);
            break;
        }

        if ch == b'?' && ends_needed > 0 {
            // A `?` print-shorthand consumes to end of line (comments
            // excepted), so — exactly like the trailing-comment case above —
            // leaving it for process_line's later `?`-conversion would
            // swallow the synthetic `end`s appended after the loop
            // (`print(args end)`). The body-START case is already handled
            // in the expansion branch below; this covers a `?` appearing
            // MID-body, after other statements. Confirmed on a real corpus
            // cart (dinkykong-0.p8.png: `if(ob.sel)rect(...) ?ob.x..","..`).
            // Convert it here: print(<args up to any trailing comment>),
            // then the pending `end`s, then any comment.
            result.extend_from_slice(b"print(");
            let mut j = i + 1;
            let mut arg_str: u8 = 0;
            while j < line.len() {
                let c = line[j];
                if arg_str != 0 {
                    if c == b'\\' && j + 1 < line.len() {
                        j += 2;
                        continue;
                    }
                    if c == arg_str {
                        arg_str = 0;
                    }
                } else if c == b'"' || c == b'\'' {
                    arg_str = c;
                } else if c == b'-' && j + 1 < line.len() && line[j + 1] == b'-' {
                    break;
                }
                j += 1;
            }
            let mut arg_end = j;
            while arg_end > i + 1 && (line[arg_end - 1] == b' ' || line[arg_end - 1] == b'\t') {
                arg_end -= 1;
            }
            result.extend_from_slice(&line[i + 1..arg_end]);
            result.push(b')');
            for _ in 0..ends_needed {
                result.extend_from_slice(b" end");
            }
            expanded_any = true;
            ends_needed = 0;
            if j < line.len() {
                result.push(b' ');
                result.extend_from_slice(&line[j..]);
            }
            break;
        }

        let kw_info: Option<(&[u8], usize, &[u8])> =
            if ch == b'i' && match_keyword_at(line, i, b"if") {
                Some((b"if", 2, b"then"))
            } else if ch == b'w' && match_keyword_at(line, i, b"while") {
                Some((b"while", 5, b"do"))
            } else {
                None
            };

        if let Some((keyword, len, separator)) = kw_info {
            let mut j = i + len;
            while j < line.len() && (line[j] == b' ' || line[j] == b'\t') {
                j += 1;
            }
            if j < line.len() && line[j] == b'(' {
                let mut depth: i32 = 0;
                let mut k = j;
                let mut paren_str: u8 = 0;
                while k < line.len() {
                    if paren_str != 0 {
                        if line[k] == b'\\' && k + 1 < line.len() {
                            k += 2;
                            continue;
                        }
                        if line[k] == paren_str {
                            paren_str = 0;
                        }
                        k += 1;
                        continue;
                    }
                    if line[k] == b'"' || line[k] == b'\'' {
                        paren_str = line[k];
                        k += 1;
                        continue;
                    }
                    if line[k] == b'(' {
                        depth += 1;
                    }
                    if line[k] == b')' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    k += 1;
                }
                if depth == 0 && k < line.len() {
                    // Whether this already has an explicit separator (`if(cond)
                    // then ...`, just with parens around the condition) can't
                    // just look at the token immediately after the first
                    // balanced `)` -- the condition can legitimately continue
                    // past it via indexing, comparison, or other operators
                    // (`if (a-b) > 0.1 then`, `if ({...})[k] then`, both real
                    // corpus patterns: sheeple-0.p8.png, kaizoleste-1.p8.png).
                    // Nor can it scan the whole rest of the line for a bare
                    // `then`/`do` anywhere (the old `has_separator_keyword`
                    // helper's approach) -- that falsely matches a separator
                    // belonging to a *nested* compound statement used AS the
                    // short-if's body, e.g. `if(a) if b then c end` (also
                    // real: build_a_jetpack-1.p8.png, tinyhawk-2.p8.png).
                    // `find_own_separator` resolves both: it scans the actual
                    // condition expression (through operators/indexing) and
                    // discounts any then/do consumed by a nested if/while/for
                    // seen along the way, so only THIS statement's own
                    // separator (if present) is reported.
                    let has_sep = find_own_separator(line, j).is_some();
                    if !has_sep {
                        let body_start = k + 1;
                        let trimmed_offset = line[body_start..]
                            .iter()
                            .position(|&b| b != b' ' && b != b'\t')
                            .unwrap_or(line.len() - body_start);
                        let rest_after_body = &line[body_start + trimmed_offset..];
                        if !rest_after_body.is_empty() && !is_continuation_body(rest_after_body) {
                            result.extend_from_slice(keyword);
                            result.push(b' ');
                            result.extend_from_slice(&line[j + 1..k]);
                            result.push(b' ');
                            result.extend_from_slice(separator);
                            result.push(b' ');
                            ends_needed += 1;
                            if rest_after_body[0] == b'?' {
                                // The short-if body is a `?` print-shorthand,
                                // which consumes to the end of the physical
                                // line. This pass runs BEFORE process_line's
                                // own `?`-handling, so if we appended a
                                // literal " end" after copying the body
                                // verbatim (the normal path below), that
                                // later pass would swallow it too --
                                // producing `print(msg,0,0,8 end)` instead
                                // of `print(msg,0,0,8) end`. Convert to
                                // print(...) here instead, so nothing `?`
                                // would consume is left after it. Confirmed
                                // against real corpus carts
                                // (pong_xmas-0.p8.png, lv-2.p8.png).
                                result.extend_from_slice(b"print(");
                                result.extend_from_slice(&rest_after_body[1..]);
                                result.push(b')');
                                for _ in 0..ends_needed {
                                    result.extend_from_slice(b" end");
                                }
                                return Some(result);
                            }
                            i = body_start;
                            continue;
                        }
                    }
                }
            }
        }

        result.push(ch);
        i += 1;
    }

    if ends_needed == 0 && !expanded_any {
        return None;
    }
    for _ in 0..ends_needed {
        result.extend_from_slice(b" end");
    }
    Some(result)
}

// Returns true if the body after `if (cond)` looks like a multi-line
// condition continuation rather than a short-if body — i.e. starts with
// a binary keyword like `or`/`and` so the real `then` is on a later line.
fn is_continuation_body(rest: &[u8]) -> bool {
    let trimmed_start = rest
        .iter()
        .position(|&b| b != b' ' && b != b'\t')
        .unwrap_or(rest.len());
    let trimmed_end = rest
        .iter()
        .rposition(|&b| b != b' ' && b != b'\t' && b != b'\r' && b != b'\n')
        .map(|p| p + 1)
        .unwrap_or(0);
    if trimmed_start >= trimmed_end {
        return false;
    }
    let trimmed = &rest[trimmed_start..trimmed_end];
    for kw in [b"or" as &[u8], b"and"] {
        if trimmed.len() >= kw.len() && &trimmed[..kw.len()] == kw {
            let after = if trimmed.len() > kw.len() {
                trimmed[kw.len()]
            } else {
                b' '
            };
            if !after.is_ascii_alphanumeric() && after != b'_' {
                return true;
            }
        }
    }
    // A body that STARTS with a binary operator (with or without a right
    // operand following on the same line) can never be a valid statement
    // -- no Lua statement begins with one -- so the parenthesized prefix
    // must be a multi-line condition whose real `then` is on a later line.
    // Two confirmed real corpus shapes: the operator alone at end of line
    // (`if(a)==` / next line `(b) then`, balloon-1.p8.png) and the
    // operator with its operand but the condition still continuing
    // (`if (abs(en.x-64))>80` / next line `or ... then`,
    // deepening-0.p8.png).
    const DANGLING_OPS: &[&[u8]] = &[
        b"==", b"~=", b"<=", b">=", b"..", b"<", b">", b"+", b"-", b"*", b"/", b"%", b"^",
    ];
    for op in DANGLING_OPS {
        if trimmed.starts_with(op) {
            return true;
        }
    }
    false
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }
    if haystack.len() < needle.len() {
        return false;
    }
    for i in 0..=haystack.len() - needle.len() {
        if &haystack[i..i + needle.len()] == needle {
            return true;
        }
    }
    false
}

const STATEMENT_KEYWORDS: &[&[u8]] = &[
    b"return", b"end", b"if", b"then", b"else", b"elseif", b"do", b"while", b"for", b"repeat",
    b"until", b"local", b"break", b"goto",
];

fn is_statement_keyword(line: &[u8], pos: usize) -> bool {
    if pos > 0 && (line[pos - 1].is_ascii_alphabetic() || line[pos - 1] == b'_') {
        return false;
    }
    for &kw in STATEMENT_KEYWORDS {
        if pos + kw.len() <= line.len() && &line[pos..pos + kw.len()] == kw {
            if pos + kw.len() < line.len() {
                let next = line[pos + kw.len()];
                if next.is_ascii_alphanumeric() || next == b'_' {
                    continue;
                }
            }
            return true;
        }
    }
    false
}

fn is_operator_keyword(line: &[u8], pos: usize) -> bool {
    let kws: &[&[u8]] = &[b"and", b"or", b"not"];
    for &kw in kws {
        if pos + kw.len() <= line.len() && &line[pos..pos + kw.len()] == kw {
            if pos + kw.len() < line.len() {
                let next = line[pos + kw.len()];
                if next.is_ascii_alphanumeric() || next == b'_' {
                    continue;
                }
            }
            return true;
        }
    }
    false
}

struct LhsResult<'a> {
    lhs: &'a [u8],
    remove_count: usize,
}

fn extract_lhs(out: &[u8]) -> LhsResult<'_> {
    let mut end = out.len();
    while end > 0 && (out[end - 1] == b' ' || out[end - 1] == b'\t') {
        end -= 1;
    }
    let mut start = end;
    while start > 0 {
        let ch = out[start - 1];
        if ch == b'.' && start >= 2 && out[start - 2] == b'.' {
            // Two consecutive dots are the concat operator, not a field
            // chain -- the LHS scan must stop here, or `"str"..x\1` walks
            // back through the `..` and splices the concat's left side into
            // the int-div rewrite (`"time:"flr(..timer/(1))`). Confirmed on
            // a real corpus cart (hakai-3.p8.png: `?"time:"..timer\1,...`).
            break;
        }
        if ch.is_ascii_alphanumeric() || ch == b'_' || ch == b'.' {
            start -= 1;
        } else if (ch == b')' || ch == b']' || ch == b'"' || ch == b'\'')
            && start < end
            && (out[start].is_ascii_alphanumeric() || out[start] == b'_')
        {
            // A closing delimiter directly abutting the identifier we've
            // already captured (no `.`/`[`/`(` connector) is value
            // JUXTAPOSITION -- two separate golfed-together statements --
            // not a call/index chain. Walking through it splices the prior
            // statement into this LHS: `e.y-=rnd"18"e.a+=rnd"6"` expanded
            // the first statement to `... - (rnd"18")` and then `e.a+=`'s
            // LHS scan walked back through `)` grabbing `rnd"18")e.a`.
            // Confirmed on a real corpus cart (praxis_fighter_x-2.p8.png).
            break;
        } else if ch == b')' {
            let mut depth: i32 = 1;
            start -= 1;
            while start > 0 && depth > 0 {
                start -= 1;
                if out[start] == b')' {
                    depth += 1;
                }
                if out[start] == b'(' {
                    depth -= 1;
                }
            }
        } else if ch == b']' {
            // Depth-tracked, unlike the old unconditional-match: a bare `[`
            // (no matching `]` yet) means we're INSIDE an open subscript's
            // index expression, and must stop there, not walk back through
            // it into the array name (`tq[o\64+1]` — lhs is `o`, not `tq[o`).
            let mut depth: i32 = 1;
            start -= 1;
            while start > 0 && depth > 0 {
                start -= 1;
                if out[start] == b']' {
                    depth += 1;
                }
                if out[start] == b'[' {
                    depth -= 1;
                }
            }
        } else if ch == b'"' || ch == b'\'' {
            // Lua's string-call sugar: `f"str"` / `f'str'` (no parens). Scan
            // back to the matching opening quote so the callee name (e.g.
            // `rnd` in `rnd"32"\1`) is still reachable on the next iteration
            // -- but ONLY if this is genuinely call sugar, i.e. an
            // identifier immediately precedes the opening quote. Otherwise
            // the string is just the tail of a completed PRIOR statement's
            // literal value coincidentally ending right where this scan
            // started (`local n="..str.."b..=x`: walking back through
            // `"..str.."`'s closing quote would otherwise splice the
            // finished `n="..str.."` assignment into the current LHS,
            // confirmed on a real corpus cart: blood_of_vladula-0.p8.png),
            // and must not be consumed at all.
            let quote = ch;
            let mut new_start = start - 1;
            while new_start > 0 {
                new_start -= 1;
                if out[new_start] == quote && (new_start == 0 || out[new_start - 1] != b'\\') {
                    break;
                }
            }
            let preceded_by_identifier = new_start > 0
                && (out[new_start - 1].is_ascii_alphanumeric() || out[new_start - 1] == b'_');
            if !preceded_by_identifier {
                break;
            }
            start = new_start;
        } else if ch == b'-' && !is_prev_value(out, start - 1) {
            // Leading unary minus (e.g. `-a\b`), distinguished from binary
            // subtraction (`a-b\c`, where the `\`'s lhs is just `b`) by
            // checking what precedes it. Without this, `-5\0` rewrote as
            // `-flr(5/(0))` instead of `flr(-5/(0))` -- the sign ended up
            // outside the call instead of inside the operand.
            start -= 1;
        } else if ch == b'#' {
            // `#` is always a prefix operator in Lua and binds tighter than
            // the binary ops these rewrites serve, so it belongs INSIDE the
            // captured operand: `#levels\n` means `flr((#levels)/n)`, but
            // stopping before the `#` produced `#flr(levels/(n))` -- length
            // of a number, a runtime error. Confirmed on a real corpus
            // cart (woodworm-0.p8.png).
            start -= 1;
        } else {
            break;
        }
    }
    LhsResult {
        lhs: &out[start..end],
        remove_count: out.len() - start,
    }
}

struct RhsResult<'a> {
    rhs: &'a [u8],
    end: usize,
}

fn extract_rhs(line: &[u8], start: usize) -> RhsResult<'_> {
    let mut i = start;
    while i < line.len() && (line[i] == b' ' || line[i] == b'\t') {
        i += 1;
    }
    let rhs_start = i;
    let mut depth: i32 = 0;
    let mut in_str: u8 = 0;

    while i < line.len() {
        let ch = line[i];
        if in_str != 0 {
            if ch == b'\\' {
                i += 1;
                if i < line.len() {
                    i += 1;
                }
                continue;
            }
            if ch == in_str {
                in_str = 0;
            }
            i += 1;
            continue;
        }
        if ch == b'"' || ch == b'\'' {
            in_str = ch;
            i += 1;
            continue;
        }
        if i + 1 < line.len()
            && ((ch == b'-' && line[i + 1] == b'-') || (ch == b'/' && line[i + 1] == b'/'))
        {
            break;
        }
        // `?` is never valid inside an expression -- it always starts a new
        // print-shorthand statement, so it's a hard stop like `;`. A `;`
        // itself likewise always terminates the statement -- without this,
        // `i+=1;return ...` captured `1;` as the RHS and spliced the
        // semicolon inside the rewritten parens (`i = i + (1;)`). Confirmed
        // on a real corpus cart (picketpuzzle-5.p8.png).
        if depth == 0 && (ch == b'?' || ch == b';') {
            break;
        }
        if ch == b'(' || ch == b'[' {
            depth += 1;
            i += 1;
            continue;
        }
        if ch == b')' || ch == b']' {
            if depth > 0 {
                depth -= 1;
            }
            i += 1;
            continue;
        }
        if depth == 0 && ch.is_ascii_alphabetic() && is_statement_keyword(line, i) {
            break;
        }
        if depth == 0 && (ch == b' ' || ch == b'\t') && i > rhs_start {
            let prev = line[i - 1];
            let prev_is_value =
                prev.is_ascii_alphanumeric() || prev == b'_' || prev == b')' || prev == b']';
            // If the word ending here is itself `and`/`or`/`not`, this
            // space separates the operator from its right-hand operand,
            // not two statements -- e.g. `axis=="x" and ox or oy` must
            // not be cut right after `and`. Confirmed against a real
            // corpus cart (celesteprogrupter-2.p8.png) where `obj.rem
            // [axis]+=axis=="x" and ox or oy` was splitting into
            // `obj.rem[axis]+(axis=="x" and)` with `ox or oy` left
            // dangling outside.
            let mut word_start = i;
            while word_start > rhs_start
                && (line[word_start - 1].is_ascii_alphanumeric() || line[word_start - 1] == b'_')
            {
                word_start -= 1;
            }
            let prev_word_is_operator_keyword = is_operator_keyword(line, word_start);
            if prev_is_value && !prev_word_is_operator_keyword {
                let mut peek = i;
                while peek < line.len() && (line[peek] == b' ' || line[peek] == b'\t') {
                    peek += 1;
                }
                if peek < line.len()
                    && (line[peek].is_ascii_alphabetic() || line[peek] == b'_')
                    && !is_operator_keyword(line, peek)
                {
                    break;
                }
            }
        }
        if depth == 0 && ch == b'=' && i > rhs_start {
            // `==` is a comparison, not the start of a glued-on assignment
            // statement -- skip the whole operator. The backward `prev`
            // check below only recognizes `==` when scanning its SECOND
            // char; the FIRST char (preceded by an ordinary identifier)
            // otherwise looks exactly like `x=...`, so an and/or ternary
            // chain with a second comparison (`a=="w"and-1or a=="e"and 1`)
            // got mis-split at the second `==`. Confirmed on a real corpus
            // cart (puzzlesofthepaladin-3.p8.png).
            if i + 1 < line.len() && line[i + 1] == b'=' {
                i += 2;
                continue;
            }
            let prev = line[i - 1];
            if prev == b'=' || prev == b'~' || prev == b'<' || prev == b'>' || prev == b'!' {
                i += 1;
                continue;
            }
            let mut lhs_end = i;
            if matches!(prev, b'+' | b'-' | b'*' | b'/' | b'%' | b'\\' | b'|' | b'&') {
                lhs_end = i - 1;
            }
            if lhs_end >= 2 {
                let pp = line[lhs_end - 1];
                if (prev == b'.' && pp == b'.')
                    || (prev == b'>' && pp == b'>')
                    || (prev == b'<' && pp == b'<')
                    || (prev == b'^' && pp == b'^')
                {
                    lhs_end -= 1;
                }
            }
            while lhs_end > rhs_start && (line[lhs_end - 1] == b' ' || line[lhs_end - 1] == b'\t') {
                lhs_end -= 1;
            }
            if lhs_end > rhs_start {
                let lhs_last = line[lhs_end - 1];
                if lhs_last.is_ascii_alphanumeric() || lhs_last == b'_' || lhs_last == b']' {
                    let mut lhs_start = lhs_end;
                    while lhs_start > rhs_start {
                        let c = line[lhs_start - 1];
                        if c.is_ascii_alphanumeric()
                            || c == b'_'
                            || c == b'.'
                            || c == b'['
                            || c == b']'
                        {
                            lhs_start -= 1;
                        } else {
                            break;
                        }
                    }
                    // A Lua identifier can't start with a digit, so a
                    // candidate like `...h%1==0` (scanning back from `1`)
                    // is just a number literal inside an expression, not
                    // the start of a new glued-together statement's LHS
                    // (e.g. real `x=1y=2`). Confirmed against a real
                    // corpus cart (pico1karena-0.p8.png: `a[...]+=
                    // h%1==0and 1or 0` was wrongly cut down to RHS `h%`).
                    let starts_with_identifier =
                        line[lhs_start].is_ascii_alphabetic() || line[lhs_start] == b'_';
                    if lhs_start > rhs_start && starts_with_identifier {
                        let mut stop = lhs_start;
                        while stop > rhs_start
                            && (line[stop - 1] == b' ' || line[stop - 1] == b'\t')
                        {
                            stop -= 1;
                        }
                        return RhsResult {
                            rhs: &line[rhs_start..stop],
                            end: lhs_start,
                        };
                    }
                }
            }
        }
        i += 1;
    }
    let mut end = i;
    while end > rhs_start && (line[end - 1] == b' ' || line[end - 1] == b'\t') {
        end -= 1;
    }
    RhsResult {
        rhs: &line[rhs_start..end],
        end: i,
    }
}

struct ExprResult<'a> {
    expr: &'a [u8],
    end: usize,
}

fn extract_simple_expr(line: &[u8], start: usize) -> ExprResult<'_> {
    let mut i = start;
    while i < line.len() && (line[i] == b' ' || line[i] == b'\t') {
        i += 1;
    }
    let expr_start = i;
    let mut depth: i32 = 0;
    let mut in_str: u8 = 0;
    while i < line.len() {
        let ch = line[i];
        if in_str != 0 {
            if ch == b'\\' {
                i += 1;
                if i < line.len() {
                    i += 1;
                }
                continue;
            }
            if ch == in_str {
                in_str = 0;
            }
            i += 1;
            continue;
        }
        if ch == b'"' || ch == b'\'' {
            in_str = ch;
            i += 1;
            continue;
        }
        // `?` is never valid inside an expression -- it always starts a new
        // print-shorthand statement, so it's a hard stop like `;`.
        if depth == 0 && ch == b'?' {
            break;
        }
        if ch == b'(' || ch == b'[' {
            depth += 1;
            i += 1;
            continue;
        }
        if ch == b')' || ch == b']' {
            if depth > 0 {
                depth -= 1;
                i += 1;
                continue;
            }
            break;
        }
        if depth == 0 {
            if matches!(
                ch,
                b'+' | b'-'
                    | b'*'
                    | b'/'
                    | b'%'
                    | b'\\'
                    | b','
                    | b';'
                    | b' '
                    | b'\t'
                    | b'<'
                    | b'>'
                    | b'='
                    | b'~'
                    | b'}'
                    | b'{'
                    | b'&'
                    | b'|'
                    | b'!'
            ) {
                break;
            }
            // `^^` (bxor) is dialect sugar applied to the peek() result, not
            // part of the address expression -- `@addr^^mask` must become
            // `peek(addr)^^mask`, confirmed against official PICO-8 (unlike
            // plain `^`, which IS real Lua exponentiation and stays part of
            // the address expression: `@addr^2` really means `peek(addr^2)`,
            // confirmed the same way -- so only the doubled form stops here).
            if ch == b'^' && i + 1 < line.len() && line[i + 1] == b'^' {
                break;
            }
            if ch.is_ascii_alphabetic() && is_statement_keyword(line, i) {
                break;
            }
        }
        i += 1;
    }
    ExprResult {
        expr: &line[expr_start..i],
        end: i,
    }
}

fn extract_bitwise_rhs(line: &[u8], start: usize) -> ExprResult<'_> {
    let mut i = start;
    while i < line.len() && (line[i] == b' ' || line[i] == b'\t') {
        i += 1;
    }
    let expr_start = i;
    let mut depth: i32 = 0;
    let mut in_str: u8 = 0;
    while i < line.len() {
        let ch = line[i];
        if in_str != 0 {
            if ch == b'\\' {
                i += 1;
                if i < line.len() {
                    i += 1;
                }
                continue;
            }
            if ch == in_str {
                in_str = 0;
            }
            i += 1;
            continue;
        }
        if ch == b'"' || ch == b'\'' {
            in_str = ch;
            i += 1;
            continue;
        }
        // A `--` (or PICO-8's C-style `//`) comment runs to end of line --
        // hard stop, or the captured RHS smuggles the comment into the
        // rewritten call and the comment eats the closing paren
        // (`1 << 3--will not swap` became `shl(1,3--will) ...`). Confirmed
        // on a real corpus cart (swap_mouse-8.p8.png).
        if i + 1 < line.len()
            && ((ch == b'-' && line[i + 1] == b'-') || (ch == b'/' && line[i + 1] == b'/'))
        {
            break;
        }
        // `?` is never valid inside an expression -- it always starts a new
        // print-shorthand statement, so it's a hard stop like `;`.
        if depth == 0 && ch == b'?' {
            break;
        }
        if ch == b'(' || ch == b'[' {
            depth += 1;
            i += 1;
            continue;
        }
        if ch == b')' || ch == b']' {
            if depth > 0 {
                depth -= 1;
                i += 1;
                continue;
            }
            break;
        }
        if depth == 0 {
            // `!` only ever occurs as the `!=` comparison in the PICO-8
            // dialect -- a hard stop, or `btn()&15!=x` captures `15!` into
            // the rewritten band() and the comparison never converts.
            // Confirmed on a real corpus cart (deepening-0.p8.png).
            if matches!(
                ch,
                b',' | b';' | b' ' | b'\t' | b'>' | b'<' | b'=' | b'&' | b'|' | b'}' | b'{' | b'!'
            ) {
                break;
            }
            // `~` alone is the unary bitwise-NOT prefix (`~x` -> `bnot(x)`)
            // and is valid RHS content -- only `~=` (inequality) is a real
            // stop. Treating a bare `~` as an unconditional stop left it
            // matching nothing (`x & ~y` has `~` as literally the first
            // character after skipping whitespace), producing an empty RHS
            // and leaving the whole `&` unconverted. Confirmed on a real
            // corpus cart (donsol8_v1-14.p8.png: `mouseb & ~mouseb_last`).
            if ch == b'~' && i + 1 < line.len() && line[i + 1] == b'=' {
                break;
            }
            if ch == b'^' && i + 1 < line.len() && line[i + 1] == b'^' {
                break;
            }
            if ch.is_ascii_alphabetic() && is_statement_keyword(line, i) {
                break;
            }
        }
        i += 1;
    }
    let mut end = i;
    while end > expr_start && (line[end - 1] == b' ' || line[end - 1] == b'\t') {
        end -= 1;
    }
    ExprResult {
        expr: &line[expr_start..end],
        end: i,
    }
}

fn try_compound_assign(line: &[u8], pos: usize, out: &mut Vec<u8>) -> Option<usize> {
    let ch = line[pos];

    let mut op: &[u8] = &[];
    let op_len: usize;
    let mut is_func = false;
    let mut func_name: &[u8] = &[];

    if ch == b'.' && pos + 2 < line.len() && line[pos + 1] == b'.' && line[pos + 2] == b'=' {
        op = b"..";
        op_len = 3;
    } else if ch == b'^' && pos + 2 < line.len() && line[pos + 1] == b'^' && line[pos + 2] == b'=' {
        is_func = true;
        func_name = b"bxor";
        op_len = 3;
    } else if (ch == b'>' || ch == b'<')
        && pos + 2 < line.len()
        && line[pos + 1] == ch
        && line[pos + 2] == b'='
    {
        is_func = true;
        func_name = if ch == b'>' { b"shr" } else { b"shl" };
        op_len = 3;
    } else if pos + 1 < line.len() && line[pos + 1] == b'=' {
        if matches!(ch, b'+' | b'-' | b'*' | b'/' | b'%' | b'\\' | b'^') {
            op_len = 2;
            // Single-char op stored via a small static set; copy below.
            // Will use a small buffer.
        } else if ch == b'|' {
            is_func = true;
            func_name = b"bor";
            op_len = 2;
        } else if ch == b'&' {
            is_func = true;
            func_name = b"band";
            op_len = 2;
        } else {
            return None;
        }
        if !is_func {
            // We'll set `op` as a 1-byte slice via static lookup.
            op = match ch {
                b'+' => b"+",
                b'-' => b"-",
                b'*' => b"*",
                b'/' => b"/",
                b'%' => b"%",
                b'\\' => b"\\",
                b'^' => b"^",
                _ => unreachable!(),
            };
        }
    } else {
        return None;
    }

    let lhs_result = extract_lhs(out);
    if lhs_result.lhs.is_empty() {
        return None;
    }
    if lhs_result.lhs.len() > 256 {
        return None;
    }
    let mut lhs_buf = [0u8; 256];
    lhs_buf[..lhs_result.lhs.len()].copy_from_slice(lhs_result.lhs);
    let lhs_len = lhs_result.lhs.len();
    let lhs = &lhs_buf[..lhs_len];

    let rhs_start = pos + op_len;
    let rhs_info = extract_rhs(line, rhs_start);
    let raw_rhs = rhs_info.rhs;
    // Recursively preprocess the RHS so operators inside get transformed.
    let processed_rhs_string: String = if !raw_rhs.is_empty() {
        let s = unsafe { core::str::from_utf8_unchecked(raw_rhs) };
        let p = preprocess(s);
        // Trim trailing newlines that preprocess() may have added between lines.
        let bytes = p.as_bytes();
        let mut end = bytes.len();
        while end > 0 && bytes[end - 1] == b'\n' {
            end -= 1;
        }
        String::from(unsafe { core::str::from_utf8_unchecked(&bytes[..end]) })
    } else {
        String::new()
    };
    let rhs: &[u8] = if raw_rhs.is_empty() {
        raw_rhs
    } else {
        processed_rhs_string.as_bytes()
    };

    if rhs.is_empty() {
        if is_func || ch == b'\\' {
            return None;
        }
        out.truncate(out.len() - lhs_result.remove_count);
        out.extend_from_slice(lhs);
        out.extend_from_slice(b" = ");
        out.extend_from_slice(lhs);
        out.push(b' ');
        out.extend_from_slice(op);
        return Some(rhs_info.end);
    }

    out.truncate(out.len() - lhs_result.remove_count);

    if is_func {
        out.extend_from_slice(lhs);
        out.extend_from_slice(b" = ");
        out.extend_from_slice(func_name);
        out.push(b'(');
        out.extend_from_slice(lhs);
        out.extend_from_slice(b", ");
        out.extend_from_slice(rhs);
        out.push(b')');
    } else if ch == b'\\' {
        out.extend_from_slice(lhs);
        out.extend_from_slice(b" = flr(");
        out.extend_from_slice(lhs);
        out.extend_from_slice(b"/(");
        out.extend_from_slice(rhs);
        out.extend_from_slice(b"))");
    } else {
        out.extend_from_slice(lhs);
        out.extend_from_slice(b" = ");
        out.extend_from_slice(lhs);
        out.push(b' ');
        out.extend_from_slice(op);
        out.extend_from_slice(b" (");
        out.extend_from_slice(rhs);
        out.push(b')');
    }

    let _ = op_len; // silence unused warning when only assigned in some branches
    Some(rhs_info.end)
}

fn try_int_div(line: &[u8], pos: usize, out: &mut Vec<u8>) -> Option<usize> {
    let lhs_result = extract_lhs(out);
    if lhs_result.lhs.is_empty() {
        return None;
    }
    if lhs_result.lhs.len() > 256 {
        return None;
    }
    let mut lhs_buf = [0u8; 256];
    lhs_buf[..lhs_result.lhs.len()].copy_from_slice(lhs_result.lhs);
    let lhs_len = lhs_result.lhs.len();
    let lhs = &lhs_buf[..lhs_len];

    let rhs_info = extract_simple_expr(line, pos + 1);
    if rhs_info.expr.is_empty() {
        return None;
    }

    out.truncate(out.len() - lhs_result.remove_count);
    out.extend_from_slice(b"flr(");
    out.extend_from_slice(lhs);
    out.extend_from_slice(b"/(");
    out.extend_from_slice(rhs_info.expr);
    out.extend_from_slice(b"))");
    Some(rhs_info.end)
}

fn try_bitwise_op(
    line: &[u8],
    pos: usize,
    op_len: usize,
    func_name: &[u8],
    out: &mut Vec<u8>,
) -> Option<usize> {
    let lhs_result = extract_lhs(out);
    if lhs_result.lhs.is_empty() {
        return None;
    }
    if lhs_result.lhs.len() > 256 {
        return None;
    }
    let mut lhs_buf = [0u8; 256];
    lhs_buf[..lhs_result.lhs.len()].copy_from_slice(lhs_result.lhs);
    let lhs_len = lhs_result.lhs.len();
    let lhs = &lhs_buf[..lhs_len];

    let rhs_info = extract_bitwise_rhs(line, pos + op_len);
    if rhs_info.expr.is_empty() {
        return None;
    }

    // Recursively preprocess the RHS so nested transforms (binary
    // literals, backslash int-div, etc.) inside it get applied -- it was
    // previously copied verbatim from the untransformed source, so e.g.
    // `fget(s) & 0b11010011` left the binary literal un-decoded, which
    // the real Lua lexer can't parse. Confirmed on real corpus carts
    // (sujurejaba-0.p8.png, spirit_solstice-9.p8.png and others:
    // `<expr> & 0b<literal> <comparison>`).
    let processed_rhs_string: String = {
        let s = unsafe { core::str::from_utf8_unchecked(rhs_info.expr) };
        let p = preprocess(s);
        let bytes = p.as_bytes();
        let mut end = bytes.len();
        while end > 0 && bytes[end - 1] == b'\n' {
            end -= 1;
        }
        String::from(unsafe { core::str::from_utf8_unchecked(&bytes[..end]) })
    };

    out.truncate(out.len() - lhs_result.remove_count);
    out.extend_from_slice(func_name);
    out.push(b'(');
    out.extend_from_slice(lhs);
    out.push(b',');
    out.extend_from_slice(processed_rhs_string.as_bytes());
    out.push(b')');
    Some(rhs_info.end)
}

fn try_peek_shortcut(line: &[u8], pos: usize, out: &mut Vec<u8>) -> Option<usize> {
    let ch = line[pos];
    let func_name: &[u8] = match ch {
        b'@' => b"peek",
        b'%' => b"peek2",
        b'$' => b"peek4",
        _ => return None,
    };

    if pos + 1 >= line.len() {
        return None;
    }
    let next = line[pos + 1];
    if !next.is_ascii_alphanumeric() && next != b'(' && next != b'_' && next != b'-' {
        return None;
    }

    let info = extract_simple_expr(line, pos + 1);
    if info.expr.is_empty() {
        return None;
    }
    // Size-golfed carts glue `@`/`%`/`$` directly onto a preceding keyword
    // with no separator (`if$24064~=0then?...`). Splicing in `peek(`/
    // `peek2(`/`peek4(` verbatim would fuse the keyword and the function
    // name into a single bad identifier for the real Lua lexer (`ifpeek4`)
    // -- insert a space when the preceding byte demands it. Confirmed
    // against a real corpus cart (redash-7.p8.png).
    if out
        .last()
        .is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_')
    {
        out.push(b' ');
    }
    out.extend_from_slice(func_name);
    out.push(b'(');
    out.extend_from_slice(info.expr);
    out.push(b')');
    Some(info.end)
}

fn match_long_open(source: &[u8], pos: usize) -> Option<usize> {
    if pos >= source.len() || source[pos] != b'[' {
        return None;
    }
    let mut level = 0usize;
    let mut i = pos + 1;
    while i < source.len() && source[i] == b'=' {
        level += 1;
        i += 1;
    }
    if i < source.len() && source[i] == b'[' {
        Some(level)
    } else {
        None
    }
}

fn match_long_close(source: &[u8], pos: usize, level: usize) -> bool {
    if pos >= source.len() || source[pos] != b']' {
        return false;
    }
    let mut i = pos + 1;
    let mut count = 0usize;
    while i < source.len() && source[i] == b'=' && count < level {
        count += 1;
        i += 1;
    }
    count == level && i < source.len() && source[i] == b']'
}

fn find_long_close(text: &[u8], level: usize) -> Option<usize> {
    let mut i = 0;
    while i < text.len() {
        if text[i] == b']' && match_long_close(text, i, level) {
            return Some(i + level + 2);
        }
        i += 1;
    }
    None
}
