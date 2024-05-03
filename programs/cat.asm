
jump start

stop_symbol:
  // Line Feed
  word 0xa

start: org 0x10
  in 0
  cmp stop_symbol
  jz break
  out 0
  jump start


break:
  halt

