container(text(""))
                                .style(move |_| container::Style {
                                    background: Some(bg_with_alpha.into()),
                                    border: Border {
                                        color: self.theme.border,
                                        width: 2.0,
                                        radius: 0.0.into(),
                                    },
                                    ..Default::default()
                                }