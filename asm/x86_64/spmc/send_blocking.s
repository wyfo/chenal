spmc_send_blocking:
	push r14
	push rbx
	sub rsp, 56
	mov r14, rsi
	mov rbx, qword ptr [rdi]
	mov rdx, qword ptr [rbx + 128]
	mov rax, rdx
	shr rax, 32
	cmp edx, eax
	je .LBB6_3
	mov rax, qword ptr [rbx + 552]
	test rax, rax
	jne .LBB6_3
	mov qword ptr [rsp + 16], rdx
	mov byte ptr [rsp + 8], 0
.LBB6_4:
	mov rsi, qword ptr [rsp + 16]
	mov rax, qword ptr [rbx + 544]
	and rax, rsi
	mov rcx, qword ptr [rbx + 528]
	#MEMBARRIER
	mov qword ptr [rcx + 8*rax], r14
	mov rcx, qword ptr [rbx + 536]
	dec rcx
	cmp rax, rcx
	jne .LBB6_6
	mov ecx, dword ptr [rbx + 544]
	or ecx, esi
	inc ecx
	movabs rax, -4294967296
	and rax, rsi
	or rax, rcx
	jmp .LBB6_7
.LBB6_6:
	lea rax, [rsi + 1]
.LBB6_7:
	xchg qword ptr [rbx + 128], rax
	mov rax, qword ptr [rbx + 552]
	test rax, rax
	jne .LBB6_8
.LBB6_10:
	mov rax, qword ptr [rbx + 384]
	test al, 1
	jne .LBB6_11
.LBB6_12:
	xor eax, eax
.LBB6_13:
	mov rdx, r14
	add rsp, 56
	pop rbx
	pop r14
	ret
.LBB6_3:
	mov dword ptr [rsp + 48], 1000000000
	lea rsi, [rbx + 128]
	lea rdi, [rsp + 8]
	lea rcx, [rsp + 24]
	call chenal::channel::Chan<T,Ch>::acquire_slot_blocking_cold
	mov eax, 1
	cmp byte ptr [rsp + 8], 0
	je .LBB6_4
	jmp .LBB6_13
.LBB6_11:
	add rbx, 384
	mov rdi, rbx
	call qword ptr [rip + aiq::queue::Queue<T,S,SP>::is_empty_locked@GOTPCREL]
	jmp .LBB6_12
.LBB6_8:
	lea rdi, [rbx + 128]
	call qword ptr [rip + <chenal::spmc::array::Array<_,C,SP> as chenal::internal::Channel>::write_slot::handle_closed@GOTPCREL]
	test al, 1
	je .LBB6_10
	mov r14, rdx
	mov eax, 1
	jmp .LBB6_13
