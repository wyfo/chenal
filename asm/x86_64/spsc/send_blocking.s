spsc_send_blocking:
	push r14
	push rbx
	sub rsp, 56
	mov r14, rsi
	mov rbx, qword ptr [rdi]
	mov rdx, qword ptr [rbx + 128]
	mov rax, rdx
	shr rax, 32
	cmp edx, eax
	je .LBB4_10
	mov rax, qword ptr [rbx + 544]
	test rax, rax
	jne .LBB4_10
	mov qword ptr [rsp + 16], rdx
	mov byte ptr [rsp + 8], 0
.LBB4_3:
	mov rsi, qword ptr [rsp + 16]
	mov rax, qword ptr [rbx + 520]
	mov rcx, qword ptr [rbx + 536]
	and rcx, rsi
	mov qword ptr [rax + 8*rcx], r14
	mov rax, qword ptr [rbx + 528]
	dec rax
	cmp rcx, rax
	jne .LBB4_5
	mov ecx, dword ptr [rbx + 536]
	or ecx, esi
	inc ecx
	movabs rax, -4294967296
	and rax, rsi
	or rax, rcx
	jmp .LBB4_6
.LBB4_5:
	lea rax, [rsi + 1]
.LBB4_6:
	xchg qword ptr [rbx + 128], rax
	mov rax, qword ptr [rbx + 544]
	test rax, rax
	jne .LBB4_12
.LBB4_7:
	mov rsi, qword ptr [rbx + 504]
	cmp rsi, 1
	jbe .LBB4_11
.LBB4_8:
	xor eax, eax
.LBB4_9:
	mov rdx, r14
	add rsp, 56
	pop rbx
	pop r14
	ret
.LBB4_10:
	mov dword ptr [rsp + 48], 1000000000
	lea rsi, [rbx + 128]
	lea rdi, [rsp + 8]
	lea rcx, [rsp + 24]
	call chenal::channel::Chan<T,Ch,SP>::acquire_slot_blocking_cold
	mov eax, 1
	cmp byte ptr [rsp + 8], 0
	je .LBB4_3
	jmp .LBB4_9
.LBB4_11:
	add rbx, 472
	mov rdi, rbx
	call qword ptr [rip + spmc_waker::SpmcWaker<_,_>::wake_unsync_cold@GOTPCREL]
	jmp .LBB4_8
.LBB4_12:
	lea rdi, [rbx + 128]
	call qword ptr [rip + <chenal::spsc::array::Array<_,C> as chenal::internal::Channel>::write_slot::handle_closed@GOTPCREL]
	test al, 1
	je .LBB4_7
	mov r14, rdx
	mov eax, 1
	jmp .LBB4_9
