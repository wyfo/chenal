chenal::channel::Chan<T,Ch>::acquire_slot_blocking_cold:
	stp x29, x30, [sp, #-64]!
	str x23, [sp, #16]
	stp x22, x21, [sp, #32]
	stp x20, x19, [sp, #48]
	mov x29, sp
	add x8, x1, #440
	mov x19, x0
	ldar x8, [x8]
	cbz x8, .LBB4_3
	strb wzr, [x19, #1]
.LBB4_2:
	mov w9, #1
	strb w9, [x19]
	ldp x20, x19, [sp, #48]
	ldr x23, [sp, #16]
	ldp x22, x21, [sp, #32]
	ldp x29, x30, [sp], #64
	ret
	adrp x23, :got:chenal::blocking::PARK_WAKER
	mov x21, x3
	mov x20, x1
	ldr x23, [x23, :got_lo12:chenal::blocking::PARK_WAKER]
	mov w0, wzr
	mov w22, w2
	b .LBB4_7
.LBB4_4:
	add x0, x20, #256
	bl spmc_waker::waker_cell::WakerCell::drop
	ldp x8, x0, [x23]
	ldr x8, [x8]
	blr x8
	add x8, x20, #288
	stp x1, x0, [x20, #256]
.LBB4_5:
	mov w0, #1
	stlr xzr, [x8]
.LBB4_6:
	add x8, x20, #440
	ldar x8, [x8]
	cbnz x8, .LBB4_1
.LBB4_7:
	add x8, x20, #128
	ldar x8, [x8]
	ldr w9, [x20, #432]
	add w8, w8, w9
	add w8, w8, #1
	cmp x8, x22
	b.ne .LBB4_16
	tbz w0, #0, .LBB4_11
	mov x0, x21
	bl chenal::blocking::Parker::park
	and w8, w0, #0xff
	cmp w8, #2
	b.ne .LBB4_20
	mov w0, wzr
	b .LBB4_6
	add x8, x20, #288
	ldar x1, [x8]
	cmp x1, #2
	b.ne .LBB4_15
	ldr x8, [x20, #256]
	ldr x9, [x23, #8]
	cmp x8, x9
	b.ne .LBB4_4
	ldr x8, [x20, #264]
	ldr x9, [x23]
	cmp x9, x8
	b.ne .LBB4_4
	add x8, x20, #288
	b .LBB4_5
.LBB4_15:
	add x0, x20, #256
	bl spmc_waker::SpmcWaker<_,_>::overwrite
	b .LBB4_6
.LBB4_16:
	orr x8, x22, x8, lsl #32
	tbz w0, #0, .LBB4_19
	add x9, x20, #288
	ldar x10, [x9]
	cmp x10, #1
	b.hi .LBB4_19
	orr x11, x10, #0x2
	casal x10, x11, [x9]
.LBB4_19:
	str x8, [x19, #8]
	strb wzr, [x19]
	ldp x20, x19, [sp, #48]
	ldr x23, [sp, #16]
	ldp x22, x21, [sp, #32]
	ldp x29, x30, [sp], #64
	ret
.LBB4_20:
	strb w8, [x19, #1]
	b .LBB4_2
