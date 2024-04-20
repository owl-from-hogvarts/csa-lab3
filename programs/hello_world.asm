
output_string:
  word 12, "hello world!"

current_char_ptr:
  word output_string

string_end:
  word 0

start:
  // while current_char_ptr != output_string + output_string.len() + 1 {
  //  out(*current_char_ptr)
  //  current_char_ptr += 1
  // }

  load current_char_ptr
  add (current_char_ptr)
  inc
  store string_end

loop:
  load current_char_ptr
  inc
  cmp string_end
  jz break
  store current_char_ptr
  load (current_char_ptr)
  out 1

break:
  halt
  


  




