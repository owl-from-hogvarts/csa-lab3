jump start
output_string:
  // length "Hello world!" 
  word 12 0x68 0x65 0x6c 0x6c 0x6f 0x20 0x77 0x6f 0x72 0x6c 0x64 0x21

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
  out 0
  jump loop

break:
  halt

