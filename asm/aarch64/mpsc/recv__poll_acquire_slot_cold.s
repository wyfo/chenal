chenal::channel::Chan<T,Ch>::poll_acquire_slot_cold:
	stp x29, x30, [sp, #-80]!
	stp x26, x25, [sp, #16]
	stp x24, x23, [sp, #32]
	stp x22, x21, [sp, #48]
	stp x20, x19, [sp, #64]
	mov x29, sp
	mov x20, x4
	mov x19, x3
	mov w8, wzr
	mov w10, #1
	mov w22, #-2
.LBB6_1:
	tbnz w10, #0, .LBB6_3
	add x9, x19, #8
	ldar x9, [x9]
	cmp x9, x20
	b.eq .LBB6_14
	add x9, x1, #256
	ldar x9, [x9]
	cbz x9, .LBB6_6
	tbnz w9, #0, .LBB6_6
	ldar x9, [x1]
	ldr x10, [x1, #432]
	sub x10, x22, x10, lsr #1
	and w9, w9, w10
	cmp x20, x9
	b.eq .LBB6_18
	tbnz w8, #0, .LBB6_13
	add x8, x1, #344
	ldar x3, [x8]
	cmp x3, #2
	b.ne .LBB6_12
	ldp x23, x21, [x2]
	ldr x8, [x1, #312]
	cmp x8, x21
	b.ne .LBB6_11
	ldr x8, [x1, #320]
	cmp x23, x8
	b.ne .LBB6_11
	add x8, x1, #344
	mov w10, wzr
	stlr xzr, [x8]
	mov w8, #1
	b .LBB6_1
.LBB6_11:
	mov x24, x0
	add x0, x1, #312
	mov x25, x1
	mov x26, x2
	bl spmc_waker::waker_cell::WakerCell::drop
	ldr x8, [x23]
	mov x0, x21
	blr x8
	mov x2, x26
	stp x1, x0, [x25, #312]
	add x9, x25, #344
	mov x0, x24
	mov x1, x25
	mov w10, wzr
	mov w8, #1
	stlr xzr, [x9]
	b .LBB6_1
.LBB6_12:
	ldp x8, x9, [x2]
	mov x21, x0
	add x0, x1, #312
	mov x23, x1
	mov x24, x2
	mov x1, x8
	mov x2, x9
	bl spmc_waker::SpmcWaker<_,_>::overwrite
	mov x2, x24
	mov x1, x23
	mov w8, w0
	mov x0, x21
	mov w10, wzr
	b .LBB6_1
	mov w8, #1
	str x8, [x0]
	ldp x20, x19, [sp, #64]
	ldp x22, x21, [sp, #48]
	ldp x24, x23, [sp, #32]
	ldp x26, x25, [sp, #16]
	ldp x29, x30, [sp], #80
	ret
.LBB6_14:
	tbz w8, #0, .LBB6_17
	add x8, x1, #344
	ldar x9, [x8]
	cmp x9, #1
	b.hi .LBB6_17
	orr x10, x9, #0x2
	casal x9, x10, [x8]
.LBB6_17:
	stp x19, x20, [x0, #8]
	b .LBB6_19
.LBB6_18:
	str xzr, [x0, #8]
.LBB6_19:
	str xzr, [x0]
	ldp x20, x19, [sp, #64]
	ldp x22, x21, [sp, #48]
	ldp x24, x23, [sp, #32]
	ldp x26, x25, [sp, #16]
	ldp x29, x30, [sp], #80
	ret
