[org 0x7c00]
[bits 16]

start_16bit:
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7c00

    push dx

    mov ah, 0x42
    mov si, disk_packet
    int 0x13
    jc disk_error

    in al, 0x92
    or al, 2
    out 0x92, al

    cli
    lgdt [gdt32_desc]

    mov eax, cr0
    or eax, 1
    mov cr0, eax
    jmp 0x08:start_32bit

disk_packet:
    db 0x10
    db 0
    dw 1 ; sectors to read(in 512 bytes)
    dw 0x7e00
    dw 0
    dq 0 ; sectors offset

disk_error:
    mov si, msg_disk_error
    call print_16bit
    jmp $

print_16bit:
    pusha
.loop:
    lodsb
    test al, al
    jz .done
    mov ah, 0x0e
    int 0x10
    jmp .loop
.done:
    popa
    ret

msg_disk_error db 'Failed to read disk', 0

[bits 32]
start_32bit:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    mov edi, 0x1000
    mov cr3, edi
    xor eax, eax
    mov ecx, 4096
    rep stosd
    mov edi, cr3

    mov dword [edi], 0x2003
    add edi, 0x1000
    mov dword [edi], 0x3003
    add edi, 0x1000
    mov dword [edi], 0x4003
    add edi, 0x1000

    mov ebx, 0x00000003
    mov ecx, 512
.set_entry:
    mov dword [edi], ebx
    add ebx, 0x1000
    add edi, 8
    loop .set_entry

    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    mov ecx, 0xc0000080
    rdmsr
    or eax, 1 << 8
    wrmsr

    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax

    lgdt [gdt64_desc]
    jmp 0x08:start_64bit

[bits 64]
start_64bit:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    mov edi, 0xb8000
    mov rax, 0x0f200f200f200f20
    mov ecx, 500
    rep stosq

    mov rdi, 0xb8000
    mov rsi, msg_64bit
    mov ah, 0x0f
    call print_64bit

    hlt
    jmp $

print_64bit:
    push rdi
    push rsi
    push rax
.print_loop:
    lodsb
    test al, al
    jz .done
    mov [rdi], ax
    add rdi, 2
    jmp .print_loop
.done:
    pop rax
    pop rsi
    pop rdi
    ret

msg_64bit db 'Igniting UNIX V11', 0

gdt32_start:
    dq 0
gdt32_code:
    dw 0xffff
    dw 0
    db 0
    db 10011010b
    db 11001111b
    db 0
gdt32_data:
    dw 0xffff
    dw 0
    db 0
    db 10010010b
    db 11001111b
    db 0
gdt32_end:

gdt32_desc:
    dw gdt32_end - gdt32_start - 1
    dd gdt32_start

gdt64_start:
    dq 0
gdt64_code:
    dw 0xFFFF
    dw 0
    db 0
    db 10011010b
    db 10101111b
    db 0
gdt64_data:
    dw 0xFFFF
    dw 0
    db 0
    db 10010010b
    db 10101111b
    db 0
gdt64_end:

gdt64_desc:
    dw gdt64_end - gdt64_start - 1
    dq gdt64_start

times 510-($-$$) db 0
dw 0xaa55