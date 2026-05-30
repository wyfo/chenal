chenal::channel::Chan<T,Ch>::acquire_slot_blocking_cold:
	stp x29, x30, [sp, #-80]!
	str x25, [sp, #16]
	stp x24, x23, [sp, #32]
	stp x22, x21, [sp, #48]
	stp x20, x19, [sp, #64]
	mov x29, sp
	adrp x24, :got:chenal::blocking::PARK_WAKER
	mov x23, x4
	mov x20, x3
	ldr x24, [x24, :got_lo12:chenal::blocking::PARK_WAKER]
	mov x21, x2
	mov x22, x1
	mov x19, x0
	mov w0, wzr
	mov w8, #1
	mov w25, #-2
	b .LBB6_2
	mov x0, x23
	bl chenal::blocking::Parker::park
	mov w8, w0
	mov w0, wzr
	and w9, w8, #0xff
	mov w8, wzr
	cmp w9, #2
	b.ne .LBB6_19
.LBB6_2:
	tbnz w8, #0, .LBB6_4
	add x8, x21, #8
	ldar x8, [x8]
	cmp x8, x20
	b.eq .LBB6_14
	add x8, x22, #256
	ldar x8, [x8]
	cbz x8, .LBB6_7
	tbnz w8, #0, .LBB6_7
	ldar x8, [x22]
	ldr x9, [x22, #432]
	sub x9, x25, x9, lsr #1
	and w8, w8, w9
	cmp x20, x8
	b.eq .LBB6_18
	tbnz w0, #0, .LBB6_1
	add x8, x22, #344
	ldar x1, [x8]
	cmp x1, #2
	b.ne .LBB6_13
	ldr x8, [x22, #312]
	ldr x9, [x24, #8]
	cmp x8, x9
	b.ne .LBB6_12
	ldr x8, [x22, #320]
	ldr x9, [x24]
	cmp x9, x8
	b.ne .LBB6_12
	add x9, x22, #344
	mov w8, wzr
	mov w0, #1
	stlr xzr, [x9]
	b .LBB6_2
.LBB6_12:
	add x0, x22, #312
	bl spmc_waker::waker_cell::WakerCell::drop
	ldp x8, x0, [x24]
	ldr x8, [x8]
	blr x8
	add x9, x22, #344
	mov w8, wzr
	stp x1, x0, [x22, #312]
	mov w0, #1
	stlr xzr, [x9]
	b .LBB6_2
.LBB6_13:
	add x0, x22, #312
	bl spmc_waker::SpmcWaker<_,_>::overwrite
	mov w8, wzr
	b .LBB6_2
.LBB6_14:
	tbz w0, #0, .LBB6_17
	add x8, x22, #344
	ldar x9, [x8]
	cmp x9, #1
	b.hi .LBB6_17
	orr x10, x9, #0x2
	casal x9, x10, [x8]
.LBB6_17:
	stp x21, x20, [x19]
	ldp x20, x19, [sp, #64]
	ldr x25, [sp, #16]
	ldp x22, x21, [sp, #48]
	ldp x24, x23, [sp, #32]
	ldp x29, x30, [sp], #80
	ret
.LBB6_18:
	strb wzr, [x19, #8]
	b .LBB6_20
.LBB6_19:
	strb w9, [x19, #8]
.LBB6_20:
	str xzr, [x19]
	ldp x20, x19, [sp, #64]
	ldr x25, [sp, #16]
	ldp x22, x21, [sp, #48]
	ldp x24, x23, [sp, #32]
	ldp x29, x30, [sp], #80
	ret
