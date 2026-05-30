chenal::channel::Chan<T,Ch>::acquire_slot_blocking_cold:
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
	add x23, x1, #256
	mov x24, sp
	mov w25, #2
.LBB14_1:
	ldr x8, [x20, #456]
	mov x12, x21
	b .LBB14_3
.LBB14_2:
	mov x12, x21
	cas x12, x10, [x20]
	cmp x12, x21
	b.eq .LBB14_19
.LBB14_3:
	ldr x10, [x20, #448]
	and x9, x8, x12
	mov x21, x12
	sub x10, x10, #1
	cmp x9, x10
	cset w10, hi
	csinv w10, w10, wzr, hs
	ands w10, w10, #0xff
	b.eq .LBB14_6
	cmp w10, #255
	b.ne .LBB14_17
	add x10, x21, #1
	mov w11, w21
	cmp x11, x21, lsr #32
	b.eq .LBB14_7
	b .LBB14_2
.LBB14_6:
	orr w10, w21, w8
	and x11, x21, #0xffffffff00000000
	add w10, w10, #1
	orr x10, x10, x11
	mov w11, w21
	cmp x11, x21, lsr #32
	b.ne .LBB14_2
.LBB14_7:
	ldr x12, [x20]
	cmp x12, x21
	b.ne .LBB14_3
	add x8, x20, #128
	ldar x12, [x8]
	ldr x8, [x20, #456]
	add w12, w12, w8
	add w12, w12, #1
	cmp x12, x11
	b.eq .LBB14_10
	bfi x10, x12, #32, #32
	b .LBB14_2
.LBB14_10:
	tbz w0, #0, .LBB14_13
	mov x0, x3
	mov x26, x3
	bl chenal::blocking::Parker::park
	and w8, w0, #0xff
	mov w0, wzr
	mov x3, x26
	cmp w8, #2
	b.eq .LBB14_1
	b .LBB14_21
	ldr x8, [sp]
	mov x26, x3
	cbnz x8, .LBB14_15
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
	b .LBB14_1
.LBB14_17:
	strb wzr, [x19, #8]
	str xzr, [x19]
	ldr x8, [sp]
	cbnz x8, .LBB14_20
.LBB14_18:
	ldp x20, x19, [sp, #144]
	ldp x22, x21, [sp, #128]
	ldp x24, x23, [sp, #112]
	ldp x26, x25, [sp, #96]
	ldp x29, x30, [sp, #80]
	add sp, sp, #160
	ret
.LBB14_19:
	ldr x8, [x20, #440]
	add x8, x8, x9, lsl #4
	stp x8, x11, [x19]
	ldr x8, [sp]
	cbz x8, .LBB14_18
.LBB14_20:
	mov x0, sp
	bl <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	b .LBB14_18
.LBB14_21:
	strb w8, [x19, #8]
	str xzr, [x19]
	ldr x8, [sp]
	cbz x8, .LBB14_18
	b .LBB14_20
	ldr x8, [sp]
	mov x19, x0
	cbnz x8, .LBB14_24
.LBB14_23:
	mov x0, x19
	bl _Unwind_Resume
	mov x0, sp
	bl <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	b .LBB14_23
	bl core::panicking::panic_in_cleanup
