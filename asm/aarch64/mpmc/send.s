mpmc_send:
	sub sp, sp, #128
	stp x29, x30, [sp, #96]
	stp x20, x19, [sp, #112]
	add x29, sp, #96
	ldr x9, [x0]
	mov w8, #1
	mov x20, sp
	stp x8, x1, [sp]
	add x8, x9, #128
	stp x8, xzr, [sp, #16]
	ldr x4, [x9, #128]
	mov w1, w4
	cmp x1, x4, lsr #32
	b.eq .LBB18_4
	ldr x10, [x9, #584]
	ldr x11, [x9, #576]
	and x10, x10, x4
	sub x11, x11, #1
	cmp x10, x11
	b.hs .LBB18_4
	add x11, x4, #1
	mov x12, x4
	casa x12, x11, [x8]
	cmp x12, x4
	mov x4, x12
	b.ne .LBB18_4
	ldr x8, [x9, #568]
	add x8, x8, x10, lsl #4
	b .LBB18_8
.LBB18_4:
	ldr x2, [x2]
	sub x0, x29, #24
	add x3, x20, #24
	mov x1, x8
	bl chenal::channel::Chan<T,Ch>::poll_acquire_slot_cold
	ldur w8, [x29, #-24]
	tbz w8, #0, .LBB18_7
	mov w0, #2
	ldr x8, [sp, #24]
	cbz x8, .LBB18_12
	b .LBB18_14
	ldp x8, x1, [x29, #-16]
.LBB18_8:
	ldr w9, [sp]
	ldr x19, [sp, #8]
	str xzr, [sp]
	tbz w9, #0, .LBB18_15
	cbz x8, .LBB18_13
	ldr x0, [sp, #16]
	str x19, [x8], #8
	stlr x1, [x8]
	add x8, x0, #312
	ldar x8, [x8]
	tbnz w8, #0, .LBB18_17
.LBB18_11:
	mov x0, xzr
	ldr x8, [sp, #24]
	cbnz x8, .LBB18_14
	mov x1, x19
	ldp x20, x19, [sp, #112]
	ldp x29, x30, [sp, #96]
	add sp, sp, #128
	ret
	mov w0, #1
	ldr x8, [sp, #24]
	cbz x8, .LBB18_12
.LBB18_14:
	mov x8, sp
	mov x20, x0
	add x0, x8, #24
	bl <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	mov x0, x20
	mov x1, x19
	ldp x20, x19, [sp, #112]
	ldp x29, x30, [sp, #96]
	add sp, sp, #128
	ret
	bl <chenal::channel::SendFuture<T,Ch,B> as core::future::future::Future>::poll::polled_after_completion
	brk #0x1
	bl <chenal::mpmc::array::Array<C,U> as chenal::internal::Channel>::write_slot::notify_receivers
	b .LBB18_11
	ldr x8, [sp, #24]
	mov x19, x0
	cbnz x8, .LBB18_20
.LBB18_19:
	mov x0, x19
	bl _Unwind_Resume
	add x0, x20, #24
	bl <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	b .LBB18_19
	bl core::panicking::panic_in_cleanup
