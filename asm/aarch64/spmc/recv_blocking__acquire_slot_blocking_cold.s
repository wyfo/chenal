chenal::channel::Chan<T,Ch,SP>::acquire_slot_blocking_cold:
	sub sp, sp, #160
	stp x29, x30, [sp, #80]
	stp x26, x25, [sp, #96]
	stp x24, x23, [sp, #112]
	stp x22, x21, [sp, #128]
	stp x20, x19, [sp, #144]
	add x29, sp, #80
	str xzr, [sp]
	adrp x22, :got:chenal::blocking::PARK_WAKER
	mov x21, x2
	ldr x22, [x22, :got_lo12:chenal::blocking::PARK_WAKER]
	mov x20, x1
	mov x19, x0
	mov w0, wzr
	add x23, x1, #296
	mov x24, sp
	mov w25, #2
.LBB10_1:
	mov x11, x21
	b .LBB10_5
	cmp x10, x11
	b.eq .LBB10_11
.LBB10_3:
	bfi x9, x11, #32, #32
.LBB10_4:
	ldr x10, [x20, #416]
	mov x11, x21
	ldr x10, [x10, x8, lsl #3]
	dmb ishld
	add x8, x20, #128
	casal x11, x9, [x8]
	cmp x11, x21
	b.eq .LBB10_18
.LBB10_5:
	ldp x10, x9, [x20, #424]
	mov x21, x11
	and x8, x9, x11
	sub x10, x10, #1
	cmp x8, x10
	b.ne .LBB10_7
	orr w9, w21, w9
	and x10, x21, #0xffffffff00000000
	add w9, w9, #1
	orr x9, x9, x10
	lsr x11, x21, #32
	mov w10, w21
	cmp x10, x11
	b.ne .LBB10_4
	b .LBB10_8
.LBB10_7:
	add x9, x21, #1
	lsr x11, x21, #32
	mov w10, w21
	cmp x10, x11
	b.ne .LBB10_4
.LBB10_8:
	add x11, x20, #128
	ldar x11, [x11]
	cmp x11, x21
	b.ne .LBB10_5
	add x12, x20, #440
	ldar x13, [x12]
	ldar x11, [x20]
	mov w11, w11
	cbz x13, .LBB10_2
	orr x13, x25, x11, lsl #2
	mov w14, #1
	casal x14, x13, [x12]
	lsr x12, x14, #2
	cmp x14, #1
	csel x11, x11, x12, eq
	cmp x10, x11
	b.ne .LBB10_3
	b .LBB10_20
.LBB10_11:
	tbz w0, #0, .LBB10_14
	mov x0, x3
	mov x26, x3
	bl chenal::blocking::Parker::park
	and w8, w0, #0xff
	mov w0, wzr
	mov x3, x26
	cmp w8, #2
	b.eq .LBB10_1
	b .LBB10_22
	ldr x8, [sp]
	mov x26, x3
	cbnz x8, .LBB10_16
	stp xzr, xzr, [x24, #8]
	stur xzr, [x24, #24]
	str x23, [sp]
	strb w25, [sp, #40]
	stp x22, x22, [x29, #-32]
	stur xzr, [x29, #-16]
	mov x0, sp
	sub x1, x29, #32
	bl aiq::wait_queue::Wait<Q,SP>::poll_wait
	mov x3, x26
	b .LBB10_1
.LBB10_18:
	str x10, [x19, #8]
	ldr x9, [sp]
	strb wzr, [x19]
	cbnz x9, .LBB10_21
.LBB10_19:
	ldp x20, x19, [sp, #144]
	ldp x22, x21, [sp, #128]
	ldp x24, x23, [sp, #112]
	ldp x26, x25, [sp, #96]
	ldp x29, x30, [sp, #80]
	add sp, sp, #160
	ret
.LBB10_20:
	strb wzr, [x19, #1]
	mov w8, #1
	ldr x9, [sp]
	strb w8, [x19]
	cbz x9, .LBB10_19
.LBB10_21:
	mov x0, sp
	bl <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	b .LBB10_19
.LBB10_22:
	strb w8, [x19, #1]
	mov w8, #1
	ldr x9, [sp]
	strb w8, [x19]
	cbz x9, .LBB10_19
	b .LBB10_21
	ldr x8, [sp]
	mov x19, x0
	cbnz x8, .LBB10_25
.LBB10_24:
	mov x0, x19
	bl _Unwind_Resume
	mov x0, sp
	bl <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	b .LBB10_24
	bl core::panicking::panic_in_cleanup
