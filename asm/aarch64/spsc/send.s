spsc_send:
	stp x29, x30, [sp, #-32]!
	stp x20, x19, [sp, #16]
	mov x29, sp
	ldr x19, [x0]
	ldr x8, [x19, #128]
	mov w9, w8
	cmp x9, x8, lsr #32
	b.eq .LBB3_8
	add x9, x19, #552
	ldar x9, [x9]
	cbnz x9, .LBB3_8
	ldr x9, [x19, #544]
	ldr x10, [x19, #528]
	and x9, x9, x8
	str x1, [x10, x9, lsl #3]
	ldr x10, [x19, #536]
	sub x10, x10, #1
	cmp x9, x10
	b.ne .LBB3_4
	ldr w9, [x19, #544]
	and x10, x8, #0xffffffff00000000
	orr w9, w8, w9
	add w9, w9, #1
	orr x9, x9, x10
	b .LBB3_5
.LBB3_4:
	add x9, x8, #1
.LBB3_5:
	str x9, [x19, #128]
	add x9, x19, #552
	dmb ish
	ldar x9, [x9]
	cbnz x9, .LBB3_13
	add x8, x19, #456
	ldar x1, [x8]
	cmp x1, #1
	b.ls .LBB3_11
	mov x0, xzr
	ldp x20, x19, [sp, #16]
	ldp x29, x30, [sp], #32
	ret
.LBB3_8:
	mov x20, x1
	ldr x1, [x2]
	add x0, x19, #128
	mov x2, x8
	bl chenal::channel::Chan<T,Ch>::poll_acquire_slot_cold
	cmp x0, #2
	b.eq .LBB3_12
	mov x8, x1
	mov x1, x20
	cbz x0, .LBB3_2
.LBB3_10:
	mov w0, #1
	ldp x20, x19, [sp, #16]
	ldp x29, x30, [sp], #32
	ret
.LBB3_11:
	add x0, x19, #424
	bl spmc_waker::SpmcWaker<_,_>::wake_unsync_cold
	mov x0, xzr
.LBB3_12:
	ldp x20, x19, [sp, #16]
	ldp x29, x30, [sp], #32
	ret
	add x0, x19, #128
	mov x1, x8
	bl <chenal::spsc::array::Array<_,C> as chenal::internal::Channel>::write_slot::handle_closed
	tbz w0, #0, .LBB3_6
	b .LBB3_10
