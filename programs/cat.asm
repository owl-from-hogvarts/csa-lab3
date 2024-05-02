
jump start

stop_symbol:
  word 0x10

start: org 0x10
  in 1
  cmp stop_symbol
  jz break
  out 1
  jump start


break:
  halt

