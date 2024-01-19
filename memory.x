MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  /* NRF52840 with Softdevice S140 7.3.0 */
  FLASH : ORIGIN = 0x00000000+156K, LENGTH = 1024K - 156K
  RAM : ORIGIN = 0x20007b08, LENGTH = 128K                 
}
