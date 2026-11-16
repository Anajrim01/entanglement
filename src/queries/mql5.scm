; MQL has no lambdas or exceptions, so a function definition is the only
; function boundary. Its name sits inside the declarator, possibly below `&`.
(function_definition declarator: (function_declarator declarator: (_) @metric.function.name)) @metric.function
(function_definition declarator: (reference_declarator (function_declarator declarator: (_) @metric.function.name))) @metric.function
(function_definition declarator: (pointer_declarator (function_declarator declarator: (_) @metric.function.name))) @metric.function

(if_statement) @metric.cognitive.if @metric.condition
(if_statement condition: (_) @metric.cognitive.condition_boundary)
(if_statement alternative: (else_clause (if_statement) @metric.cognitive.else_if))
(if_statement alternative: (else_clause (compound_statement)) @metric.cognitive.else)
(if_statement alternative: (else_clause (expression_statement)) @metric.cognitive.else)
(while_statement) @metric.cognitive.loop @metric.condition
(while_statement condition: (_) @metric.cognitive.condition_boundary)
(do_statement) @metric.cognitive.loop @metric.condition
(for_statement) @metric.cognitive.loop
(for_statement condition: (_)) @metric.condition
(conditional_expression) @metric.cognitive.if @metric.condition
(conditional_expression condition: (_) @metric.cognitive.condition_boundary)
(switch_statement) @metric.cognitive.multiway
; `default` is a case_statement without a value and adds no path.
(case_statement value: (_)) @metric.condition
(binary_expression operator: "&&" @metric.cognitive.logical_and) @metric.cognitive.logical_expression @metric.logical_condition
(binary_expression operator: "||" @metric.cognitive.logical_or) @metric.cognitive.logical_expression @metric.logical_condition
(parenthesized_expression) @metric.cognitive.parentheses
(goto_statement) @metric.cognitive.labeled_jump

(comment) @comment
_ @syntax.node
