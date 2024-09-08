/* Specify the memory areas */
MEMORY
{
  RAM (xrw)       : ORIGIN = 0x20000000, LENGTH = 192K
  FLASH (rx)      : ORIGIN = 0x8000000, LENGTH = 1024K
  TCMRAM (rw)     : ORIGIN = 0x10000000, LENGTH = 64K
}
