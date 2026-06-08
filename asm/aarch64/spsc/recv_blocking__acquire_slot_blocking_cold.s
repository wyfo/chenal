chenal::channel::Chan<T,Ch>::acquire_slot_blocking_cold:
	stp x29, x30, [sp, #-64]!
	stp x24, x23, [sp, #16]
	stp x22, x21, [sp, #32]
	stp x20, x19, [sp, #48]
	mov x29, sp
	add x23, x1, #424
	ldar x8, [x1]
	mov x20, x1
	ldar x9, [x23]
	mov x19, x0
	mov w22, w2
	cbz x9, .LBB1_5
	mov w0, wzr
	ldar x8, [x20]
	mov w9, #2
	mov w10, #1
	bfi x9, x8, #2, #32
	mov w8, w8
	casal x10, x9, [x23]
	lsr x9, x10, #2
	cmp x10, #1
	csel x8, x8, x9, eq
	cmp x22, x8
	b.ne .LBB1_18
	strb wzr, [x19, #1]
.LBB1_4:
	mov w9, #1
	strb w9, [x19]
	ldp x20, x19, [sp, #48]
	ldp x22, x21, [sp, #32]
	ldp x24, x23, [sp, #16]
	ldp x29, x30, [sp], #64
	ret
	adrp x24, :got:chenal::blocking::PARK_WAKER
	mov x21, x3
	mov w0, wzr
	ldr x24, [x24, :got_lo12:chenal::blocking::PARK_WAKER]
	b .LBB1_9
.LBB1_6:
	add x0, x20, #296
	bl spmc_waker::waker_cell::WakerCell::drop
	ldp x8, x0, [x24]
	ldr x8, [x8]
	blr x8
	add x8, x20, #328
	stp x1, x0, [x20, #296]
.LBB1_7:
	mov w0, #1
	stlr xzr, [x8]
.LBB1_8:
	add x9, x20, #424
	ldar x8, [x20]
	ldar x9, [x9]
	cbnz x9, .LBB1_2
.LBB1_9:
	mov w8, w8
	cmp x22, x8
	b.ne .LBB1_18
	tbz w0, #0, .LBB1_13
	mov x0, x21
	bl chenal::blocking::Parker::park
	and w8, w0, #0xff
	cmp w8, #2
	b.ne .LBB1_22
	mov w0, wzr
	b .LBB1_8
	add x8, x20, #328
	ldar x1, [x8]
	cmp x1, #2
	b.ne .LBB1_17
	ldr x8, [x20, #296]
	ldr x9, [x24, #8]
	cmp x8, x9
	b.ne .LBB1_6
	ldr x8, [x20, #304]
	ldr x9, [x24]
	cmp x9, x8
	b.ne .LBB1_6
	add x8, x20, #328
	b .LBB1_7
.LBB1_17:
	add x0, x20, #296
	bl spmc_waker::SpmcWaker<_,_>::overwrite
	b .LBB1_8
.LBB1_18:
	orr x8, x22, x8, lsl #32
	tbz w0, #0, .LBB1_21
	ldr x9, [x20, #328]
	cmp x9, #1
	b.hi .LBB1_21
	orr x10, x9, #0x2
	add x11, x20, #328
	cas x9, x10, [x11]
.LBB1_21:
	str x8, [x19, #8]
	strb wzr, [x19]
	ldp x20, x19, [sp, #48]
	ldp x22, x21, [sp, #32]
	ldp x24, x23, [sp, #16]
	ldp x29, x30, [sp], #64
	ret
.LBB1_22:
	strb w8, [x19, #1]
	b .LBB1_4
