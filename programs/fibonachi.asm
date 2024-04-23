
first:
  word 1

second:
  word 2

next:
  word 0

max:
  word 4_000_000

sum:
  // 2 is second member of fibonachi series
  // so it is always included
  word 2

start:
  load first
  add second
  store next
  cmp max
  jcs break
  andi 0x1 // to check for even numbers
  jzc next
  load sum
  add next
  store sum

next:
  load second
  store first
  load next
  store second
  jump start

break:
  load sum
  out 0x1
  halt
