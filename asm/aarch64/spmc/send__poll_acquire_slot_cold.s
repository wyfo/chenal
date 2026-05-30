chenal::channel::Chan<T,Ch>::poll_acquire_slot_cold:
	stp x29, x30, [sp, #-64]!
	str x23, [sp, #16]
	stp x22, x21, [sp, #32]
	stp x20, x19, [sp, #48]
	mov x29, sp
	add x8, x0, #440
	ldar x8, [x8]
	cbz x8, .LBB4_3
	mov w8, #1
	mov x0, x8
	ldp x20, x19, [sp, #48]
	ldr x23, [sp, #16]
	ldp x22, x21, [sp, #32]
	ldp x29, x30, [sp], #64
	ret
	mov w9, wzr
	mov w20, w2
	mov w8, #2
	add x10, x0, #128
	ldar x10, [x10]
	ldr w11, [x0, #432]
	add w10, w10, w11
	add w10, w10, #1
	cmp x10, x20
	b.ne .LBB4_13
	tbnz w9, #0, .LBB4_2
	add x9, x0, #288
	ldar x3, [x9]
	cmp x3, #2
	b.ne .LBB4_12
	ldp x21, x19, [x1]
	ldr x9, [x0, #256]
	cmp x9, x19
	b.ne .LBB4_11
	ldr x9, [x0, #264]
	cmp x21, x9
	b.ne .LBB4_11
	add x9, x0, #288
	stlr xzr, [x9]
	mov w9, #1
.LBB4_10:
	add x10, x0, #440
	ldar x10, [x10]
	cbz x10, .LBB4_4
	b .LBB4_17
.LBB4_11:
	mov x22, x0
	add x0, x0, #256
	mov x23, x1
	bl spmc_waker::waker_cell::WakerCell::drop
	ldr x8, [x21]
	mov x0, x19
	blr x8
	mov x9, x0
	mov x10, x1
	add x11, x22, #288
	mov x0, x22
	mov w8, #2
	mov x1, x23
	stp x10, x9, [x22, #256]
	mov w9, #1
	stlr xzr, [x11]
	b .LBB4_10
.LBB4_12:
	ldp x8, x2, [x1]
	mov x19, x0
	add x0, x0, #256
	mov x21, x1
	mov x1, x8
	bl spmc_waker::SpmcWaker<_,_>::overwrite
	mov x1, x21
	mov w9, w0
	mov x0, x19
	mov w8, #2
	b .LBB4_10
.LBB4_13:
	orr x1, x20, x10, lsl #32
	tbz w9, #0, .LBB4_16
	add x8, x0, #288
	ldar x9, [x8]
	cmp x9, #1
	b.hi .LBB4_16
	orr x10, x9, #0x2
	casal x9, x10, [x8]
.LBB4_16:
	mov x8, xzr
	mov x0, x8
	ldp x20, x19, [sp, #48]
	ldr x23, [sp, #16]
	ldp x22, x21, [sp, #32]
	ldp x29, x30, [sp], #64
	ret
.LBB4_17:
	mov w8, #1
	mov x0, x8
	ldp x20, x19, [sp, #48]
	ldr x23, [sp, #16]
	ldp x22, x21, [sp, #32]
	ldp x29, x30, [sp], #64
	ret
