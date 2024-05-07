
jump pre_start

stop_symbol:
  // Line Feed
  word 0xa

pre_start:
  // skip string length
  in 0

start: org 0x10
  in 0
  cmp stop_symbol
  jz break
  out 0
  jump start


break:
  halt

