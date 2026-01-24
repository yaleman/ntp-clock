MEMORY
{
    /* Raspberry Pi Pico 2 W (RP2350) */
    FLASH : ORIGIN = 0x10000000, LENGTH = 0x400000
    RAM : ORIGIN = 0x20000000, LENGTH = 0x82000
}

SECTIONS {
    .start_block : ALIGN(4)
    {
        KEEP(*(.start_block));
    } > FLASH

} INSERT AFTER .vector_table;

SECTIONS {
    .boot_info : ALIGN(4)
    {
        KEEP(*(.boot_info));
    } > FLASH

} INSERT AFTER .start_block;

_stext = ADDR(.boot_info) + SIZEOF(.boot_info);

SECTIONS {
    .bi_entries : ALIGN(4)
    {
        __bi_entries_start = .;
        KEEP(*(.bi_entries));
        . = ALIGN(4);
        __bi_entries_end = .;
    } > FLASH
} INSERT AFTER .text;

SECTIONS {
    .flash_end : {
        __flash_binary_end = .;
    } > FLASH
} INSERT AFTER .uninit;
