// CAUTION! PROGRAM IS NOT REENTRANT

jump start

first:
  word 1

second:
  word 2

next_number:
  word 0

max:
  word 4_000_000

sum:
  // 2 is second member of fibonachi series
  // so it is always included
  word 2

minus_one:
  word 0xff_ff_ff_ff

shifted_number:
  word 0

digit_counter: 
  word 0

digits_ptr:
  word 0

digits_base_ptr:
  word digits

digits:
  word 0x30 // 0
  word 0x31 // 1
  word 0x32 // 2
  word 0x33 // 3
  word 0x34 // 4
  word 0x35 // 5
  word 0x36 // 6
  word 0x37 // 7
  word 0x38 // 8
  word 0x39 // 9
  word 0x61 // a
  word 0x62 // b
  word 0x63 // c
  word 0x64 // d
  word 0x65 // e
  word 0x66 // f

// there are eight hex digits in 32 bit number
output_buffer_size:
  word 8

output_buffer_ptr:
  word output_buffer

output_buffer:
  word 0
  word 0
  word 0
  word 0

  word 0
  word 0
  word 0
  word 0

start:
  load first
  add second
  store next_number
  cmp max
  jcs to_string
  andi 0x1 // to check for even numbers
  jzc next
  load sum
  add next_number
  store sum

next:
  load second
  store first
  load next_number
  store second
  jump start

to_string:
  load sum
  store shifted_number

digit_to_string:
  load shifted_number
  // take 4 bits to form a digit
  andi 0b1111
  add digits_base_ptr
  store digits_ptr
  load (digits_ptr)
  store (output_buffer_ptr)

  load shifted_number

  shift_right
  shift_right
  shift_right
  shift_right

  store shifted_number

  load output_buffer_ptr
  inc
  store output_buffer_ptr

  load digit_counter
  inc
  store digit_counter

  cmp output_buffer_size
  jzc digit_to_string

output_buffer_loop:
  load output_buffer_ptr
  add minus_one
  store output_buffer_ptr
  load (output_buffer_ptr)
  out 0

  load digit_counter
  add minus_one
  store digit_counter
  jzc output_buffer_loop

  halt
