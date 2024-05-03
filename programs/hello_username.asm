jump start

ask_string_ptr:
  word ask_string

ask_string:
  word 19
  word 0x57 0x68 0x61 0x74 0x20 0x69 0x73 0x20 0x79 0x6f 0x75 0x72 0x20 0x6e 0x61 0x6d 0x65 0x3f 0xa

greet_string_ptr:
  word greet_string

greet_string:
  word 7
  word 0x48 0x65 0x6c 0x6c 0x6f 0x2c 0x20

minus_one:
  word 0xff_ff_ff_ff

get_input_ptr:
  word get_input

output_user_name_ptr:
  word output_user_name

break_ptr:
  word break

input_past_the_end:
  word 0

start:
  load stack_ptr
  add minus_one
  store stack_ptr

  load get_input_ptr
  store (stack_ptr)

  load stack_ptr
  add minus_one
  store stack_ptr

  load ask_string_ptr
  store (stack_ptr)
  jump output_string

get_input:
  load input_buffer_base_ptr
  store input_buffer_ptr
  in 0
  store (input_buffer_ptr)
  inc
  add input_buffer_ptr
  store input_past_the_end

  load input_buffer_ptr
  inc
  store input_buffer_ptr

get_input_loop:
  load input_buffer_ptr
  cmp input_past_the_end
  jz greet
  
  in 0
  store (input_buffer_ptr)
  
  load input_buffer_ptr
  inc
  store input_buffer_ptr
  jump get_input_loop

greet:
  load stack_ptr
  add minus_one
  store stack_ptr
  
  load output_user_name_ptr
  store (stack_ptr)

  load stack_ptr
  add minus_one
  store stack_ptr

  load greet_string_ptr
  store (stack_ptr)

  jump output_string

output_user_name:
  load stack_ptr
  add minus_one
  store stack_ptr

  load break_ptr
  store (stack_ptr)

  load stack_ptr
  add minus_one
  store stack_ptr

  load input_buffer_base_ptr
  store (stack_ptr)
  jump output_string

break:
  halt
  
output_string_past_the_end:
  word 0

output_string_ptr:
  word 0

output_string:
  load (stack_ptr)
  store output_string_ptr
  load (output_string_ptr) // get length
  inc // account for length occupying one cell
  add output_string_ptr // compute past the end pointer
  store output_string_past_the_end

  // pop
  load stack_ptr
  inc
  store stack_ptr
  
  // proceed to first character
  load output_string_ptr
  inc 
  store output_string_ptr

output_string_loop:
  load output_string_ptr
  cmp output_string_past_the_end
  jz return
  load (output_string_ptr)
  out 0

  load output_string_ptr
  inc 
  store output_string_ptr
  jump output_string_loop

return_address:
  word 0

return:
  load (stack_ptr)
  store return_address
  load stack_ptr
  inc
  store stack_ptr
  jump (return_address)


stack_ptr:
  word 0x0

input_buffer_length:
  word 256

input_buffer_base_ptr:
  word 0x500

input_buffer_ptr:
  word 0
