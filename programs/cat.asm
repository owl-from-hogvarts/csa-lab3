
stop_symbol:
  word 0x10

start:
  in 1
  cmp stop_symbol
  jz break
  out 1
  jump start


break:
  halt

