MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  /* NRF52840 with Softdevice S140 7.3.0 */
  /* Softdeviceがメモリマップの先頭に配置されるので、Rust製バイナリのFlash/RAMの先頭位置をその後ろに配置する。 */  
  /* See "s140_nrf52_7.3.0_release-note.pdf" p.5 "SoftDevice properties" */
  /* Flash: 156.0kB(0x27000 bytes). */
  /* RAMのLENGTHは128KBになっているが、実際には256KBある。 本来は256K-(SoftDeviceの使用量)とするべき*/  
  FLASH : ORIGIN = 0x00000000+156K, LENGTH = 1024K - 156K
  RAM : ORIGIN = 0x20007b08, LENGTH = 128K                 
}
