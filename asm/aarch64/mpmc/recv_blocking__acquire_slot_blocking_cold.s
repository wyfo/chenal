chenal::channel::Chan<T,Ch>::acquire_slot_blocking_cold:
	sub sp, sp, #176
	stp x29, x30, [sp, #80]
	str x27, [sp, #96]
	stp x26, x25, [sp, #112]
	stp x24, x23, [sp, #128]
	stp x22, x21, [sp, #144]
	stp x20, x19, [sp, #160]
	add x29, sp, #80
	str xzr, [sp]
	adrp x24, :got:chenal::blocking::PARK_WAKER
	mov x20, x3
	ldr x24, [x24, :got_lo12:chenal::blocking::PARK_WAKER]
	mov x22, x2
	mov x21, x1
	mov x19, x0
	mov w0, wzr
	add x23, x1, #312
	mov x25, sp
	mov w26, #-2
	mov w27, #2
.LBB9_1:
	mov x9, x22
	b .LBB9_3
.LBB9_2:
	ldr x9, [x21, #128]
	cmp x9, x22
	b.eq .LBB9_7
.LBB9_3:
	ldr x8, [x21, #456]
	ldr x10, [x21, #440]
	mov x22, x9
	and x9, x8, x9
	add x8, x10, x9, lsl #4
	add x10, x8, #8
	ldar x10, [x10]
	cmp x10, x22
	b.ne .LBB9_2
	ldr x8, [x8]
	dmb ishld
	ldr x10, [x21, #448]
	sub x10, x10, #1
	cmp x9, x10
	b.ne .LBB9_6
	ldr x9, [x21, #456]
	orr x9, x22, x9
	add x10, x9, #1
	add x11, x21, #128
	mov x9, x22
	casal x9, x10, [x11]
	cmp x9, x22
	b.ne .LBB9_3
	b .LBB9_16
.LBB9_6:
	add x10, x22, #1
	add x11, x21, #128
	mov x9, x22
	casal x9, x10, [x11]
	cmp x9, x22
	b.ne .LBB9_3
	b .LBB9_16
.LBB9_7:
	add x8, x21, #256
	ldar x8, [x8]
	cbz x8, .LBB9_10
	tbnz w8, #0, .LBB9_10
	ldr x8, [x21]
	ldr x9, [x21, #456]
	sub x9, x26, x9, lsr #1
	and w8, w8, w9
	cmp x22, x8
	b.eq .LBB9_18
	tbz w0, #0, .LBB9_13
	mov x0, x20
	bl chenal::blocking::Parker::park
	and w8, w0, #0xff
	mov w0, wzr
	cmp w8, #2
	b.eq .LBB9_1
	b .LBB9_19
	ldr x8, [sp]
	cbnz x8, .LBB9_15
	stp xzr, xzr, [x25, #8]
	stur xzr, [x25, #24]
	str x23, [sp]
	strb w27, [sp, #40]
	stp x24, x24, [x29, #-32]
	stur xzr, [x29, #-16]
	mov x0, sp
	sub x1, x29, #32
	bl aiq::wait_queue::Wait<Q,SP>::poll_wait
	b .LBB9_1
.LBB9_16:
	str x8, [x19, #8]
	ldr x8, [sp]
	strb wzr, [x19]
	cbnz x8, .LBB9_21
.LBB9_17:
	ldp x20, x19, [sp, #160]
	ldr x27, [sp, #96]
	ldp x22, x21, [sp, #144]
	ldp x24, x23, [sp, #128]
	ldp x26, x25, [sp, #112]
	ldp x29, x30, [sp, #80]
	add sp, sp, #176
	ret
.LBB9_18:
	strb wzr, [x19, #1]
	b .LBB9_20
.LBB9_19:
	strb w8, [x19, #1]
.LBB9_20:
	mov w9, #1
	ldr x8, [sp]
	strb w9, [x19]
	cbz x8, .LBB9_17
	mov x0, sp
	bl <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	b .LBB9_17
	ldr x8, [sp]
	mov x19, x0
	cbnz x8, .LBB9_24
.LBB9_23:
	mov x0, x19
	bl _Unwind_Resume
	mov x0, sp
	bl <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	b .LBB9_23
	bl core::panicking::panic_in_cleanup
