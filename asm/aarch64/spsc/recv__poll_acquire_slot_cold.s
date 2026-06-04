chenal::channel::Chan<T,Ch>::poll_acquire_slot_cold:
	stp x29, x30, [sp, #-80]!
	str x25, [sp, #16]
	stp x24, x23, [sp, #32]
	stp x22, x21, [sp, #48]
	stp x20, x19, [sp, #64]
	mov x29, sp
	add x22, x0, #424
	ldar x9, [x0]
	mov w21, w2
	ldar x8, [x22]
	cbz x8, .LBB1_8
	mov w8, wzr
.LBB1_2:
	ldar x9, [x0]
	mov w10, #2
	mov w11, #1
	mov w19, #1
	bfi x10, x9, #2, #32
	mov w9, w9
	casal x11, x10, [x22]
	lsr x10, x11, #2
	cmp x11, #1
	csel x9, x9, x10, eq
	cmp x21, x9
	b.eq .LBB1_7
.LBB1_3:
	orr x1, x21, x9, lsl #32
	tbz w8, #0, .LBB1_6
	add x8, x0, #328
	ldar x9, [x8]
	cmp x9, #1
	b.hi .LBB1_6
	orr x10, x9, #0x2
	casal x9, x10, [x8]
.LBB1_6:
	mov x19, xzr
.LBB1_7:
	mov x0, x19
	ldp x20, x19, [sp, #64]
	ldr x25, [sp, #16]
	ldp x22, x21, [sp, #48]
	ldp x24, x23, [sp, #32]
	ldp x29, x30, [sp], #80
	ret
	mov w19, #2
	mov w9, w9
	cmp x21, x9
	b.ne .LBB1_3
	tbnz w8, #0, .LBB1_18
	add x8, x0, #328
	ldar x3, [x8]
	cmp x3, #2
	b.ne .LBB1_17
	ldp x23, x20, [x1]
	ldr x8, [x0, #296]
	cmp x8, x20
	b.ne .LBB1_16
	ldr x8, [x0, #304]
	cmp x23, x8
	b.ne .LBB1_16
	add x8, x0, #328
	stlr xzr, [x8]
	mov w8, #1
.LBB1_15:
	add x10, x0, #424
	ldar x9, [x0]
	ldar x10, [x10]
	cbz x10, .LBB1_9
	b .LBB1_2
.LBB1_16:
	mov x24, x0
	add x0, x0, #296
	mov x25, x1
	bl spmc_waker::waker_cell::WakerCell::drop
	ldr x8, [x23]
	mov x0, x20
	blr x8
	mov x8, x0
	mov x9, x1
	add x10, x24, #328
	mov x0, x24
	mov x1, x25
	stp x9, x8, [x24, #296]
	mov w8, #1
	stlr xzr, [x10]
	b .LBB1_15
.LBB1_17:
	ldp x8, x2, [x1]
	mov x20, x0
	add x0, x0, #296
	mov x23, x1
	mov x1, x8
	bl spmc_waker::SpmcWaker<_,_>::overwrite
	mov x1, x23
	mov w8, w0
	mov x0, x20
	b .LBB1_15
	b .LBB1_7
