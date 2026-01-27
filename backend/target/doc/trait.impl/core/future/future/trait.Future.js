;(() => {
	var implementors = Object.fromEntries([
		[
			'tower_governor',
			[
				[
					'impl&lt;F, E&gt; <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/future/future/trait.Future.html" title="trait core::future::future::Future">Future</a> for <a class="struct" href="tower_governor/struct.ResponseFuture.html" title="struct tower_governor::ResponseFuture">ResponseFuture</a>&lt;F&gt;<div class="where">where\n    F: <a class="trait" href="https://doc.rust-lang.org/1.91.1/core/future/future/trait.Future.html" title="trait core::future::future::Future">Future</a>&lt;Output = <a class="enum" href="https://doc.rust-lang.org/1.91.1/core/result/enum.Result.html" title="enum core::result::Result">Result</a>&lt;Response&lt;Body&gt;, E&gt;&gt;,</div>',
				],
			],
		],
	])
	if (window.register_implementors) {
		window.register_implementors(implementors)
	} else {
		window.pending_implementors = implementors
	}
})()
//{"start":57,"fragment_lengths":[724]}
