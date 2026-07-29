function sanitized_line(source,    out, i, c, next_two, hashes, j, closing) {
  out = ""
  i = 1

  while (i <= length(source)) {
    c = substr(source, i, 1)
    next_two = substr(source, i, 2)

    if (block_comment_depth > 0) {
      if (next_two == "/*") {
        block_comment_depth++
        i += 2
      } else if (next_two == "*/") {
        block_comment_depth--
        i += 2
      } else {
        i++
      }
      continue
    }

    if (raw_string_hashes >= 0) {
      closing = "\""
      for (j = 0; j < raw_string_hashes; j++) closing = closing "#"
      if (substr(source, i, length(closing)) == closing) {
        raw_string_hashes = -1
        i += length(closing)
      } else {
        i++
      }
      continue
    }

    if (quoted_string) {
      if (c == "\\") {
        i += 2
      } else if (c == "\"") {
        quoted_string = 0
        i++
      } else {
        i++
      }
      continue
    }

    if (next_two == "//") break
    if (next_two == "/*") {
      block_comment_depth = 1
      i += 2
      continue
    }

    if (c == "r") {
      hashes = 0
      j = i + 1
      while (substr(source, j, 1) == "#") {
        hashes++
        j++
      }
      if (substr(source, j, 1) == "\"") {
        raw_string_hashes = hashes
        i = j + 1
        continue
      }
    }

    if (c == "\"") {
      quoted_string = 1
      i++
      continue
    }

    if (c == "'" && substr(source, i + 2, 1) == "'") {
      i += 3
      continue
    }
    if (c == "'" && substr(source, i + 1, 1) == "\\" && substr(source, i + 3, 1) == "'") {
      i += 4
      continue
    }

    out = out c
    i++
  }

  return out
}

function brace_delta(source,    i, c, delta) {
  delta = 0
  for (i = 1; i <= length(source); i++) {
    c = substr(source, i, 1)
    if (c == "{") delta++
    if (c == "}") delta--
  }
  return delta
}

BEGIN {
  if (scanner_mode == "") scanner_mode = "inline_test_ranges"
  depth = 0
  pending_cfg_test = 0
  in_inline_test = 0
  inline_test_floor = 0
  block_comment_depth = 0
  raw_string_hashes = -1
  quoted_string = 0
}

{
  code = sanitized_line($0)

  if (scanner_mode == "ignored_attributes") {
    if (code ~ /^[[:space:]]*#\[[[:space:]]*ignore([[:space:]]*=[[:space:]]*)?[[:space:]]*\]/) {
      print "direct\t" source_path
      next
    }

    if (!in_cfg_attr &&
        code ~ /^[[:space:]]*#\[[[:space:]]*cfg_attr[[:space:]]*\(/) {
      in_cfg_attr = 1
      cfg_attr_bracket_depth = 0
      cfg_attr_has_ignore = 0
    }
    if (in_cfg_attr) {
      if (code ~ /(^|[^[:alnum:]_])ignore([^[:alnum:]_]|$)/) {
        cfg_attr_has_ignore = 1
      }
      for (i = 1; i <= length(code); i++) {
        c = substr(code, i, 1)
        if (c == "[") cfg_attr_bracket_depth++
        if (c == "]") cfg_attr_bracket_depth--
      }
      if (cfg_attr_bracket_depth <= 0) {
        if (cfg_attr_has_ignore) print "conditional\t" source_path
        in_cfg_attr = 0
        cfg_attr_bracket_depth = 0
        cfg_attr_has_ignore = 0
      }
    }
    next
  }

  depth_before = depth
  started_inline_test = 0

  if (!in_inline_test && code ~ /#\[[[:space:]]*cfg[[:space:]]*\([[:space:]]*test[[:space:]]*\)[[:space:]]*\]/) {
    pending_cfg_test = 1
  }

  module_line = code ~ /(^|[[:space:]])(pub([[:space:]]*\([^)]*\))?[[:space:]]+)?mod[[:space:]]+[[:alnum:]_]+[[:space:]]*\{/
  if (!in_inline_test && pending_cfg_test && module_line) {
    in_inline_test = 1
    inline_test_floor = depth_before + 1
    pending_cfg_test = 0
    started_inline_test = 1
  } else if (!in_inline_test && pending_cfg_test && code !~ /^[[:space:]]*$/ && code !~ /^[[:space:]]*#/) {
    pending_cfg_test = 0
  }

  depth_after = depth + brace_delta(code)
  if (in_inline_test && !started_inline_test && depth_after >= inline_test_floor) {
    print source_path "\t" FNR
  }

  depth = depth_after
  if (in_inline_test && depth_after < inline_test_floor) {
    in_inline_test = 0
    inline_test_floor = 0
  }
}
