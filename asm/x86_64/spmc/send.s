spmc_send:
	push r14
	push rbx
	push rax
	mov rax, rdx
	mov r14, rsi
	mov rbx, qword ptr [rdi]
	mov rdx, qword ptr [rbx + 128]
	mov rcx, rdx
	shr rcx, 32
	cmp edx, ecx
	je .LBB6_10
	mov rcx, qword ptr [rbx + 552]
	test rcx, rcx
	jne .LBB6_10
.LBB6_2:
	mov rax, qword ptr [rbx + 544]
	and rax, rdx
	mov rcx, qword ptr [rbx + 528]
	#MEMBARRIER
	mov qword ptr [rcx + 8*rax], r14
	mov rcx, qword ptr [rbx + 536]
	dec rcx
	cmp rax, rcx
	jne .LBB6_4
	mov ecx, dword ptr [rbx + 544]
	or ecx, edx
	inc ecx
	movabs rax, -4294967296
	and rax, rdx
	or rax, rcx
	jmp .LBB6_5
.LBB6_4:
	lea rax, [rdx + 1]
.LBB6_5:
	xchg qword ptr [rbx + 128], rax
	mov rax, qword ptr [rbx + 552]
	test rax, rax
	jne .LBB6_15
.LBB6_6:
	mov rax, qword ptr [rbx + 384]
	test al, 1
	jne .LBB6_13
.LBB6_7:
	xor eax, eax
.LBB6_8:
.LBB6_9:
	mov rdx, r14
	add rsp, 8
	pop rbx
	pop r14
	ret
.LBB6_10:
	mov rsi, qword ptr [rax]
	lea rdi, [rbx + 128]
	call chenal::channel::Chan<T,Ch>::poll_acquire_slot_cold
	cmp rax, 2
	je .LBB6_8
	test rax, rax
	je .LBB6_2
.LBB6_12:
	mov eax, 1
	jmp .LBB6_9
.LBB6_13:
	add rbx, 384
	mov rdi, rbx
	call qword ptr [rip + aiq::queue::Queue<T,S,SP>::is_empty_locked@GOTPCREL]
	jmp .LBB6_7
.LBB6_15:
	lea rdi, [rbx + 128]
	mov rsi, rdx
	call qword ptr [rip + <chenal::spmc::array::Array<_,C,SP> as chenal::internal::Channel>::write_slot::handle_closed@GOTPCREL]
	test al, 1
	je .LBB6_6
	mov r14, rdx
	jmp .LBB6_12
