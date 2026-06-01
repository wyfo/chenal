mpmc_send:
	sub sp, sp, #128
	stp x29, x30, [sp, #96]
	stp x20, x19, [sp, #112]
	add x29, sp, #96
	ldr x10, [x0]
	mov w8, #1
	mov x20, sp
	stp x8, x1, [sp]
	add x9, x10, #128
	stp x9, xzr, [sp, #16]
	ldr x4, [x10, #128]
	mov w8, w4
	cmp x8, x4, lsr #32
	b.eq .LBB18_4
	ldr x11, [x10, #584]
	ldr x12, [x10, #576]
	and x11, x11, x4
	sub x12, x12, #1
	cmp x11, x12
	b.hs .LBB18_4
	add x12, x4, #1
	mov x13, x4
	cas x13, x12, [x9]
	cmp x13, x4
	mov x4, x13
	b.ne .LBB18_4
	ldr x9, [x10, #568]
	ldr x0, [sp, #16]
	str xzr, [sp]
	add x9, x9, x11, lsl #4
	b .LBB18_10
.LBB18_4:
	ldr x2, [x2]
	sub x0, x29, #24
	add x3, x20, #24
	mov x1, x9
	bl chenal::channel::Chan<T,Ch,SP>::poll_acquire_slot_cold
	ldur w8, [x29, #-24]
	tbz w8, #0, .LBB18_7
	mov w0, #2
	ldr x8, [sp, #24]
	cbz x8, .LBB18_12
	b .LBB18_14
	ldp x9, x8, [x29, #-16]
	ldr w10, [sp]
	ldr x1, [sp, #8]
	str xzr, [sp]
	tbz w10, #0, .LBB18_15
	cbz x9, .LBB18_13
	ldr x0, [sp, #16]
.LBB18_10:
	dmb ishld
	dmb ish
	str x1, [x9], #8
	stlr x8, [x9]
	add x9, x0, #312
	ldar x9, [x9]
	tbnz w9, #0, .LBB18_17
.LBB18_11:
	mov x0, xzr
	ldr x8, [sp, #24]
	cbnz x8, .LBB18_14
	ldp x20, x19, [sp, #112]
	ldp x29, x30, [sp, #96]
	add sp, sp, #128
	ret
	mov w0, #1
	ldr x8, [sp, #24]
	cbz x8, .LBB18_12
.LBB18_14:
	mov x8, sp
	mov x19, x0
	mov x20, x1
	add x0, x8, #24
	bl <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	mov x0, x19
	mov x1, x20
	ldp x20, x19, [sp, #112]
	ldp x29, x30, [sp, #96]
	add sp, sp, #128
	ret
	bl <chenal::channel::SendFuture<T,Ch,B,SP> as core::future::future::Future>::poll::polled_after_completion
	brk #0x1
	mov x1, x8
	bl <chenal::mpmc::array::Array<C,_> as chenal::internal::Channel>::write_slot::notify_receivers
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
